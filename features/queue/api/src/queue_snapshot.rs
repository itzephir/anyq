use crate::{Error, Result, Task, TaskState, Topic, TopicQueue};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueueSnapshot {
    pub topics: Vec<Topic>,
    pub tasks: Vec<Task>,
}

impl QueueSnapshot {
    pub fn new(mut topics: Vec<Topic>, mut tasks: Vec<Task>) -> Result<Self> {
        topics.sort_by(|a, b| a.name.cmp(&b.name));
        tasks.sort_by_key(Task::sequence);
        let names: BTreeSet<_> = topics.iter().map(|topic| topic.name.as_str()).collect();
        let task_names: BTreeSet<_> = tasks.iter().map(Task::name).collect();
        let sequences: BTreeSet<_> = tasks.iter().map(Task::sequence).collect();

        if names.len() != topics.len()
            || task_names.len() != tasks.len()
            || sequences.len() != tasks.len()
            || tasks.iter().any(|task| {
                !TopicQueue::STATES.contains(&task.state())
                    || task
                        .topics()
                        .iter()
                        .any(|name| !names.contains(name.as_str()))
            })
        {
            return Err(Error::Invalid(
                "inconsistent queue observation snapshot".into(),
            ));
        }

        Ok(Self { topics, tasks })
    }

    pub fn count(&self, state: TaskState) -> u64 {
        self.tasks
            .iter()
            .filter(|task| task.state() == state)
            .count() as u64
    }

    pub fn topic_count(&self, topic: &str, state: TaskState) -> u64 {
        self.tasks
            .iter()
            .filter(|task| task.state() == state && task.topics().iter().any(|name| name == topic))
            .count() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multi_topic_owner_counts_once_and_preserves_order_and_start_preferences() {
        let owner = Task::restore(
            1,
            "`owner`".into(),
            TaskState::Running,
            vec!["cpu".into(), "repo".into()],
        )
        .unwrap();
        let stopped = Task::restore(2, "".into(), TaskState::Queued, vec!["cpu".into()])
            .unwrap()
            .with_start_policy(false, true);
        let topics = ["repo", "empty", "cpu"].map(|name| Topic::restore(name.into(), 2).unwrap());

        let snapshot = QueueSnapshot::new(topics.into(), vec![stopped.clone(), owner]).unwrap();

        assert_eq!(snapshot.count(TaskState::Running), 1);
        assert_eq!(snapshot.topic_count("cpu", TaskState::Running), 1);
        assert_eq!(snapshot.topic_count("repo", TaskState::Running), 1);
        assert_eq!(snapshot.topic_count("empty", TaskState::Queued), 0);
        assert_eq!(snapshot.tasks[1], stopped);
        assert_eq!(snapshot.tasks[0].name(), "`owner`");
        assert_eq!(snapshot.topics[0].name, "cpu");
    }

    #[test]
    fn non_current_tasks_and_missing_memberships_are_rejected() {
        let registered = Task::restore(1, "job".into(), TaskState::Registered, vec![]).unwrap();
        let queued = Task::restore(2, "job".into(), TaskState::Queued, vec!["cpu".into()]).unwrap();

        assert!(QueueSnapshot::new(vec![], vec![registered]).is_err());
        assert!(QueueSnapshot::new(vec![], vec![queued]).is_err());
    }
}
