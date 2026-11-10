use crate::{lookup_topic::lookup_topic, ports::Store};
use anyq_queue_api::{
    Metadata, QueuePolicy, ReplaceTopicMetadata, ReplaceTopicMetadataRequest, Result,
};

pub struct ReplaceTopicMetadataInteractor<S> {
    store: S,
    policy: QueuePolicy,
}

impl<S> ReplaceTopicMetadataInteractor<S> {
    pub fn new(store: S, policy: QueuePolicy) -> Self {
        Self { store, policy }
    }
}

impl<S: Store> ReplaceTopicMetadata for ReplaceTopicMetadataInteractor<S> {
    fn replace(&mut self, request: ReplaceTopicMetadataRequest) -> Result<Metadata> {
        self.store.transaction(|records| {
            let mut entity = lookup_topic(records, &request.topic, self.policy.autocreate)?;

            entity.metadata = request.metadata;

            records.topics().put_topic(&entity)?;

            Ok(entity.metadata.clone())
        })
    }
}
