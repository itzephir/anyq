use crate::{lookup_task::lookup_task, lookup_topic::lookup_topic, ports::Store};
use anyq_queue_api::{AddTaskToTopic, Result, Task};

pub struct AddTaskToTopicInteractor<S> {
    store: S,
}

impl<S> AddTaskToTopicInteractor<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S: Store> AddTaskToTopic for AddTaskToTopicInteractor<S> {
    fn add(&mut self, task: &str, topic: &str) -> Result<Task> {
        self.store.transaction(|records| {
            let mut task = lookup_task(records, task)?;
            lookup_topic(records, topic)?;

            task.add_topic(topic)?;
            records.tasks().save_task(&task)?;

            Ok(task)
        })
    }
}
