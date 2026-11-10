use crate::{lookup_topic::lookup_topic, ports::Store};
use anyq_queue_api::{Metadata, QueuePolicy, Result, SetTopicMetadata, SetTopicMetadataRequest};

pub struct SetTopicMetadataInteractor<S> {
    store: S,
    policy: QueuePolicy,
}

impl<S> SetTopicMetadataInteractor<S> {
    pub fn new(store: S, policy: QueuePolicy) -> Self {
        Self { store, policy }
    }
}

impl<S: Store> SetTopicMetadata for SetTopicMetadataInteractor<S> {
    fn set(&mut self, request: SetTopicMetadataRequest) -> Result<Metadata> {
        self.store.transaction(|records| {
            let mut entity = lookup_topic(records, &request.topic, self.policy.autocreate)?;

            entity.metadata.set(&request.key, request.value)?;

            records.topics().put_topic(&entity)?;

            Ok(entity.metadata.clone())
        })
    }
}
