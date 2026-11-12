use crate::ports::*;
use anyq_queue_api::*;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Memory {
    pub topics: Vec<Topic>,
    pub tasks: Vec<Task>,
    pub events: Vec<EventSnapshot>,
    pub work: Vec<ManagedWork>,
    pub intents: std::collections::BTreeSet<String>,
    pub receipts: std::collections::BTreeMap<(String, String), CompletionReceipt>,
    pub directories: std::collections::BTreeMap<String, ExecutionDirectory>,
    pub fail_save: bool,
    pub fail_task: Option<String>,
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
    fn tasks(&mut self, filter: &TaskFilter) -> Result<Vec<Task>> {
        Ok(self
            .tasks
            .iter()
            .filter(|task| matches_filter(task, filter))
            .cloned()
            .collect())
    }

    fn task_names(&mut self) -> Result<Vec<String>> {
        let mut names: Vec<_> = self
            .tasks
            .iter()
            .map(|task| task.name().to_owned())
            .collect();
        names.sort();

        Ok(names)
    }

    fn task(&mut self, name: &str) -> Result<Option<Task>> {
        Ok(self.tasks.iter().find(|task| task.name() == name).cloned())
    }

    fn insert_task(&mut self, name: &str) -> Result<Task> {
        if self.tasks.iter().any(|task| task.name() == name) {
            return Err(Error::Conflict("duplicate task".into()));
        }

        let task = Task::restore(
            self.tasks.len() as i64 + 1,
            name.into(),
            TaskState::Registered,
            Vec::new(),
        )?;
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
        if self.fail_save || self.fail_task.as_deref() == Some(task.name()) {
            return Err(Error::Storage("injected save failure".into()));
        }

        Ok(())
    }

    fn count_tasks(&mut self, filter: &TaskFilter) -> Result<u64> {
        Ok(self
            .tasks
            .iter()
            .filter(|task| matches_filter(task, filter))
            .count() as u64)
    }
}

fn matches_filter(task: &Task, filter: &TaskFilter) -> bool {
    task.state() == filter.state
        && filter
            .topic
            .as_ref()
            .is_none_or(|topic| task.topics().contains(topic))
        && filter
            .before_sequence
            .is_none_or(|sequence| task.sequence() < sequence)
}

impl Clock for Memory {
    fn now_millis(&self) -> i64 {
        1234
    }
}

impl EventRecords for Memory {
    fn append(&mut self, event: &EventSnapshot) -> Result<()> {
        self.events.push(event.clone());

        Ok(())
    }

    fn save_directory(&mut self, task: &str, directory: &ExecutionDirectory) -> Result<()> {
        self.directories.insert(task.into(), directory.clone());

        Ok(())
    }

    fn directory(&mut self, task: &str) -> Result<Option<ExecutionDirectory>> {
        Ok(self.directories.get(task).cloned())
    }
}

impl WorkRecords for Memory {
    fn managed(&mut self, owner: Option<&str>, task: Option<&str>) -> Result<Vec<ManagedWork>> {
        Ok(self
            .work
            .iter()
            .filter(|work| {
                owner.is_none_or(|owner| owner == work.owner)
                    && task.is_none_or(|task| task == work.task)
            })
            .cloned()
            .collect())
    }

    fn save_managed(&mut self, work: &ManagedWork) -> Result<()> {
        self.work
            .retain(|item| item.owner != work.owner || item.action_id != work.action_id);
        self.work.push(work.clone());

        Ok(())
    }

    fn completion_requested(&mut self, task: &str) -> Result<bool> {
        Ok(self.intents.contains(task))
    }

    fn request_completion(&mut self, task: &str) -> Result<()> {
        self.intents.insert(task.into());
        for work in &mut self.work {
            if work.task == task && work.state.blocks_completion() {
                work.cancellation_requested = true;
            }
        }

        Ok(())
    }

    fn receipt(&mut self, owner: &str, action_id: &str) -> Result<Option<CompletionReceipt>> {
        Ok(self
            .receipts
            .get(&(owner.into(), action_id.into()))
            .cloned())
    }

    fn save_receipt(
        &mut self,
        owner: &str,
        action_id: &str,
        receipt: &CompletionReceipt,
    ) -> Result<()> {
        self.receipts
            .insert((owner.into(), action_id.into()), receipt.clone());

        Ok(())
    }
}

impl UnitOfWork for Memory {
    fn work(&mut self) -> &mut dyn WorkRecords {
        self
    }

    fn clock(&self) -> &dyn Clock {
        self
    }

    fn caller_directory(&self) -> Option<&ExecutionDirectory> {
        None
    }

    fn events(&mut self) -> &mut dyn EventRecords {
        self
    }

    fn topics(&mut self) -> &mut dyn TopicRecords {
        self
    }

    fn tasks(&mut self) -> &mut dyn TaskRecords {
        self
    }
}
