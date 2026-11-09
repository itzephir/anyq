use crate::{lookup_task::lookup_task, lookup_topic::lookup_topic, ports::Store};
use anyq_queue_api::AutoCreatePolicy;
use anyq_queue_api::{AddTaskToTopic, Result, Task};

pub struct AddTaskToTopicInteractor<S> {
    store: S,
    policy: AutoCreatePolicy,
}

impl<S> AddTaskToTopicInteractor<S> {
    pub fn new(store: S) -> Self {
        Self::with_policy(store, AutoCreatePolicy::default())
    }

    pub fn with_policy(store: S, policy: AutoCreatePolicy) -> Self {
        Self { store, policy }
    }
}

impl<S: Store> AddTaskToTopic for AddTaskToTopicInteractor<S> {
    fn add(&mut self, task: &str, topic: &str) -> Result<Task> {
        self.store.transaction(|records| {
            let mut task = lookup_task(records, task, self.policy)?;
            let topic = lookup_topic(records, topic, self.policy)?;

            task.link_topic(&topic)?;
            records.tasks().save_task(&task)?;

            Ok(task)
        })
    }
}
