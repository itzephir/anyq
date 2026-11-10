use crate::record_event::topic_event;
use crate::{lookup_topic::lookup_topic, ports::Store};
use anyq_queue_api::QueueEventKind;
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

            let previous = entity.metadata.clone();
            entity.metadata = request.metadata;

            records.topics().put_topic(&entity)?;

            if previous != entity.metadata {
                topic_event(records, QueueEventKind::TopicMetadataChanged, &entity)?;
            }

            Ok(entity.metadata.clone())
        })
    }
}
