use crate::admit_queued_tasks::admit_queued_tasks;
use crate::{
    lookup_topic::lookup_topic_value,
    ports::{Store, TaskFilter},
};
use anyq_queue_api::AutoCreatePolicy;
use anyq_queue_api::{Result, SetTopicCapacity, TaskState, Topic};

pub struct SetTopicCapacityInteractor<S> {
    store: S,
    policy: AutoCreatePolicy,
}

impl<S> SetTopicCapacityInteractor<S> {
    pub fn new(store: S) -> Self {
        Self::with_policy(store, AutoCreatePolicy::default())
    }

    pub fn with_policy(store: S, policy: AutoCreatePolicy) -> Self {
        Self { store, policy }
    }
}

impl<S: Store> SetTopicCapacity for SetTopicCapacityInteractor<S> {
    fn set_capacity(&mut self, topic: Topic) -> Result<Topic> {
        let mut topic = Topic::restore(topic.name, topic.capacity)?;

        self.store.transaction(|records| {
            let existing = lookup_topic_value(records, &topic.name, self.policy)?;
            topic.autostart = existing.autostart;
            topic.metadata = existing.metadata;
            let running = records.tasks().count_tasks(&TaskFilter {
                topic: Some(topic.name.clone()),
                state: TaskState::Running,
                before_sequence: None,
            })?;

            topic.require_capacity_for(running)?;
            records.topics().put_topic(&topic)?;

            admit_queued_tasks(records)?;

            Ok(topic)
        })
    }
}
