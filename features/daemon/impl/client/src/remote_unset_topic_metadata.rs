use crate::Channel;
use anyq_daemon_api::{QueueRequest, QueueResponse};
use anyq_queue_api::{Metadata, Result, UnsetTopicMetadata, UnsetTopicMetadataRequest};

pub struct RemoteUnsetTopicMetadata {
    channel: Channel,
}

impl RemoteUnsetTopicMetadata {
    pub fn new(channel: Channel) -> Self {
        Self { channel }
    }
}

impl UnsetTopicMetadata for RemoteUnsetTopicMetadata {
    fn unset(&mut self, request: UnsetTopicMetadataRequest) -> Result<Metadata> {
        match self.channel.request(QueueRequest::UnsetTopicMetadata {
            topic: request.topic,
            key: request.key,
        })? {
            QueueResponse::Metadata(value) => value.try_into(),
            _ => Err(Channel::unexpected()),
        }
    }
}
