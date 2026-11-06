use crate::lookup_topic::lookup_topic;
use crate::ports::*;
use anyq_queue_api::*;

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
        self.store.transaction(|records| {
            let topic = Topic::new(topic.name, topic.capacity)?;
            lookup_topic(records, &topic.name)?;
            records.topics().put_topic(&topic)?;
            Ok(topic)
        })
    }
}
