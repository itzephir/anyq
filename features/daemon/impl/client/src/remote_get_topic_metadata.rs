use crate::Channel;
use anyq_daemon_api::{QueueRequest, QueueResponse};
use anyq_queue_api::{GetTopicMetadata, GetTopicMetadataRequest, MetadataValue, Result};

pub struct RemoteGetTopicMetadata {
    channel: Channel,
}

impl RemoteGetTopicMetadata {
    pub fn new(channel: Channel) -> Self {
        Self { channel }
    }
}

impl GetTopicMetadata for RemoteGetTopicMetadata {
    fn get(&mut self, request: GetTopicMetadataRequest) -> Result<MetadataValue> {
        match self.channel.request(QueueRequest::GetTopicMetadata {
            topic: request.topic,
            key: request.key,
        })? {
            QueueResponse::Metadata(value) => value.try_into(),
            _ => Err(Channel::unexpected()),
        }
    }
}
