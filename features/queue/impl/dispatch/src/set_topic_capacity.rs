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
        let topic = Topic::restore(topic.name, topic.capacity)?;

        self.store.transaction(|records| {
            lookup_topic_value(records, &topic.name, self.policy)?;
            let running = records.tasks().count_tasks(&TaskFilter {
                topic: topic.name.clone(),
                state: TaskState::Running,
                before_sequence: None,
            })?;

            topic.require_capacity_for(running)?;
            records.topics().put_topic(&topic)?;

            Ok(topic)
        })
    }
}
