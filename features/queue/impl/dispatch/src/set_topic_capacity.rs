use crate::{
    lookup_topic::lookup_topic,
    ports::{Store, TaskFilter},
};
use anyq_queue_api::{Result, SetTopicCapacity, TaskState, Topic};

pub struct SetTopicCapacityInteractor<S> {
    store: S,
}

impl<S> SetTopicCapacityInteractor<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S: Store> SetTopicCapacity for SetTopicCapacityInteractor<S> {
    fn set_capacity(&mut self, topic: Topic) -> Result<Topic> {
        let topic = Topic::new(topic.name, topic.capacity)?;

        self.store.transaction(|records| {
            lookup_topic(records, &topic.name)?;
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
