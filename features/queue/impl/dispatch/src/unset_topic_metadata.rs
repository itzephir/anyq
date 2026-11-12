use crate::{lookup_topic::lookup_topic, ports::Store};
use anyq_queue_api::{
    Metadata, QueuePolicy, Result, UnsetTopicMetadata, UnsetTopicMetadataRequest,
};

pub struct UnsetTopicMetadataInteractor<S> {
    store: S,
    policy: QueuePolicy,
}

impl<S> UnsetTopicMetadataInteractor<S> {
    pub fn new(store: S, policy: QueuePolicy) -> Self {
        Self { store, policy }
    }
}

impl<S: Store> UnsetTopicMetadata for UnsetTopicMetadataInteractor<S> {
    fn unset(&mut self, request: UnsetTopicMetadataRequest) -> Result<Metadata> {
        self.store.transaction(|records| {
            let mut entity = lookup_topic(records, &request.topic, self.policy.autocreate)?;

            entity.metadata.unset(&request.key)?;

            records.topics().put_topic(&entity)?;

            Ok(entity.metadata.clone())
        })
    }
}
