use crate::Channel;
use anyq_daemon_api::{QueueRequest, QueueResponse};
use anyq_queue_api::{Metadata, ReplaceTopicMetadata, ReplaceTopicMetadataRequest, Result};

pub struct RemoteReplaceTopicMetadata {
    channel: Channel,
}

impl RemoteReplaceTopicMetadata {
    pub fn new(channel: Channel) -> Self {
        Self { channel }
    }
}

impl ReplaceTopicMetadata for RemoteReplaceTopicMetadata {
    fn replace(&mut self, request: ReplaceTopicMetadataRequest) -> Result<Metadata> {
        match self.channel.request(QueueRequest::ReplaceTopicMetadata {
            topic: request.topic,
            metadata: request.metadata.into(),
        })? {
            QueueResponse::Metadata(value) => value.try_into(),
            _ => Err(Channel::unexpected()),
        }
    }
}
