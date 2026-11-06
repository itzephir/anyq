use crate::lookup_topic::lookup_topic;
use crate::ports::*;
use anyq_queue_api::*;

pub struct CreateConsumerGroupInteractor<S> {
    store: S,
}
impl<S> CreateConsumerGroupInteractor<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}
impl<S: Store> CreateConsumerGroup for CreateConsumerGroupInteractor<S> {
    fn create(&mut self, group: ConsumerGroup) -> Result<ConsumerGroup> {
        self.store.transaction(|records| {
            validate_name(&group.name)?;
            lookup_topic(records, &group.topic)?;
            if !records.groups().groups(&group.topic)?.contains(&group) {
                records.groups().insert_group(&group)?;
                records.deliveries().seed_group(&group)?;
            }
            Ok(group)
        })
    }
}
