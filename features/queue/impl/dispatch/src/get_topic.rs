use crate::lookup_topic::lookup_topic;
use crate::ports::*;
use anyq_queue_api::*;

pub struct GetTopicInteractor<S> {
    store: S,
}
impl<S> GetTopicInteractor<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}
impl<S: Store> GetTopic for GetTopicInteractor<S> {
    fn get(&mut self, name: &str) -> Result<Topic> {
        self.store
            .transaction(|records| lookup_topic(records, name))
    }
}
