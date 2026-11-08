use crate::{
    lookup_topic::lookup_topic,
    ports::{Store, TaskFilter},
};
use anyq_queue_api::{GetTopicQueue, Result, TopicQueue};

pub struct GetTopicQueueInteractor<S> {
    store: S,
}

impl<S> GetTopicQueueInteractor<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S: Store> GetTopicQueue for GetTopicQueueInteractor<S> {
    fn get(&mut self, name: &str) -> Result<TopicQueue> {
        self.store.transaction(|records| {
            let topic = lookup_topic(records, name)?;
            let mut tasks = Vec::new();

            for state in TopicQueue::STATES {
                tasks.extend(records.tasks().tasks(&TaskFilter {
                    topic: name.into(),
                    state,
                    before_sequence: None,
                })?);
            }

            Ok(TopicQueue::new(topic, tasks))
        })
    }
}
