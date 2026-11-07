use crate::ports::*;
use anyq_queue_api::*;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Memory {
    pub topics: Vec<Topic>,
    pub tasks: Vec<Task>,
    pub fail_save: bool,
}

impl TopicRecords for Memory {
    fn topics(&mut self) -> Result<Vec<Topic>> {
        Ok(self.topics.clone())
    }

    fn topic(&mut self, name: &str) -> Result<Option<Topic>> {
        Ok(self.topics.iter().find(|topic| topic.name == name).cloned())
    }

    fn put_topic(&mut self, topic: &Topic) -> Result<()> {
        self.topics.retain(|item| item.name != topic.name);
        self.topics.push(topic.clone());

        Ok(())
    }
}

impl TaskRecords for Memory {
    fn task(&mut self, name: &str) -> Result<Option<Task>> {
        Ok(self.tasks.iter().find(|task| task.name() == name).cloned())
    }

    fn insert_task(&mut self, name: &str) -> Result<Task> {
        if self.tasks.iter().any(|task| task.name() == name) {
            return Err(Error::Conflict("duplicate task".into()));
        }

        let task = Task::new(self.tasks.len() as i64 + 1, name.into())?;
        self.tasks.push(task.clone());

        Ok(task)
    }

    fn save_task(&mut self, task: &Task) -> Result<()> {
        let target = self
            .tasks
            .iter_mut()
            .find(|item| item.sequence() == task.sequence())
            .ok_or_else(|| Error::Storage("missing task".into()))?;
        *target = task.clone();

        // Fail after the mutation so the Store test exercises actual rollback.
        if self.fail_save {
            return Err(Error::Storage("injected save failure".into()));
        }

        Ok(())
    }

    fn count_tasks(&mut self, filter: &TaskFilter) -> Result<u64> {
        Ok(self
            .tasks
            .iter()
            .filter(|task| {
                task.state() == filter.state
                    && task.topics().contains(&filter.topic)
                    && filter
                        .before_sequence
                        .is_none_or(|sequence| task.sequence() < sequence)
            })
            .count() as u64)
    }
}

impl UnitOfWork for Memory {
    fn topics(&mut self) -> &mut dyn TopicRecords {
        self
    }

    fn tasks(&mut self) -> &mut dyn TaskRecords {
        self
    }
}
