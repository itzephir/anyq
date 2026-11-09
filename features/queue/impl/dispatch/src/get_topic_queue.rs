use crate::{
    lookup_topic::lookup_topic,
    ports::{Store, TaskFilter},
};
use anyq_queue_api::AutoCreatePolicy;
use anyq_queue_api::{GetTopicQueue, Result, TopicQueue};

pub struct GetTopicQueueInteractor<S> {
    store: S,
    policy: AutoCreatePolicy,
}

impl<S> GetTopicQueueInteractor<S> {
    pub fn new(store: S) -> Self {
        Self::with_policy(store, AutoCreatePolicy::default())
    }

    pub fn with_policy(store: S, policy: AutoCreatePolicy) -> Self {
        Self { store, policy }
    }
}

impl<S: Store> GetTopicQueue for GetTopicQueueInteractor<S> {
    fn get(&mut self, name: &str) -> Result<TopicQueue> {
        self.store.transaction(|records| {
            let topic = lookup_topic(records, name, self.policy)?;
            let mut tasks = Vec::new();

            for state in TopicQueue::STATES {
                tasks.extend(records.tasks().tasks(&TaskFilter {
                    topic: topic.name.clone(),
                    state,
                    before_sequence: None,
                })?);
            }

            Ok(TopicQueue::new(topic, tasks))
        })
    }
}
