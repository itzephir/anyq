use crate::Channel;
use anyq_daemon_api::{QueueRequest, QueueResponse};
use anyq_queue_api::{GetTaskMetadata, GetTaskMetadataRequest, MetadataValue, Result};

pub struct RemoteGetTaskMetadata {
    channel: Channel,
}

impl RemoteGetTaskMetadata {
    pub fn new(channel: Channel) -> Self {
        Self { channel }
    }
}

impl GetTaskMetadata for RemoteGetTaskMetadata {
    fn get(&mut self, request: GetTaskMetadataRequest) -> Result<MetadataValue> {
        match self.channel.request(QueueRequest::GetTaskMetadata {
            task: request.task,
            key: request.key,
        })? {
            QueueResponse::Metadata(value) => value.try_into(),
            _ => Err(Channel::unexpected()),
        }
    }
}
