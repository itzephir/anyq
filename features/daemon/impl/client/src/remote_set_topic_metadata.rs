use crate::Channel;
use anyq_daemon_api::{QueueRequest, QueueResponse};
use anyq_queue_api::{Metadata, Result, SetTopicMetadata, SetTopicMetadataRequest};

pub struct RemoteSetTopicMetadata {
    channel: Channel,
}

impl RemoteSetTopicMetadata {
    pub fn new(channel: Channel) -> Self {
        Self { channel }
    }
}

impl SetTopicMetadata for RemoteSetTopicMetadata {
    fn set(&mut self, request: SetTopicMetadataRequest) -> Result<Metadata> {
        match self.channel.request(QueueRequest::SetTopicMetadata {
            topic: request.topic,
            key: request.key,
            value: request.value.into(),
        })? {
            QueueResponse::Metadata(value) => value.try_into(),
            _ => Err(Channel::unexpected()),
        }
    }
}
