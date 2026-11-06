use crate::lookup_topic::lookup_topic;
use crate::ports::*;
use anyq_queue_api::*;

pub struct ListConsumerGroupsInteractor<S> {
    store: S,
}
impl<S> ListConsumerGroupsInteractor<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}
impl<S: Store> ListConsumerGroups for ListConsumerGroupsInteractor<S> {
    fn list(&mut self, topic: &str) -> Result<Vec<ConsumerGroup>> {
        self.store.transaction(|records| {
            lookup_topic(records, topic)?;
            records.groups().groups(topic)
        })
    }
}
