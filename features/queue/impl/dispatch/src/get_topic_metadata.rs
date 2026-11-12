use crate::{lookup_topic::lookup_topic, ports::Store};
use anyq_queue_api::{
    GetTopicMetadata, GetTopicMetadataRequest, MetadataValue, QueuePolicy, Result,
};

pub struct GetTopicMetadataInteractor<S> {
    store: S,
    policy: QueuePolicy,
}

impl<S> GetTopicMetadataInteractor<S> {
    pub fn new(store: S, policy: QueuePolicy) -> Self {
        Self { store, policy }
    }
}

impl<S: Store> GetTopicMetadata for GetTopicMetadataInteractor<S> {
    fn get(&mut self, request: GetTopicMetadataRequest) -> Result<MetadataValue> {
        self.store.transaction(|records| {
            let entity = lookup_topic(records, &request.topic, self.policy.autocreate)?;

            entity.metadata.get(request.key.as_deref()).cloned()
        })
    }
}
