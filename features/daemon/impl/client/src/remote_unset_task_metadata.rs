use crate::Channel;
use anyq_daemon_api::{QueueRequest, QueueResponse};
use anyq_queue_api::{Metadata, Result, UnsetTaskMetadata, UnsetTaskMetadataRequest};

pub struct RemoteUnsetTaskMetadata {
    channel: Channel,
}

impl RemoteUnsetTaskMetadata {
    pub fn new(channel: Channel) -> Self {
        Self { channel }
    }
}

impl UnsetTaskMetadata for RemoteUnsetTaskMetadata {
    fn unset(&mut self, request: UnsetTaskMetadataRequest) -> Result<Metadata> {
        match self.channel.request(QueueRequest::UnsetTaskMetadata {
            task: request.task,
            key: request.key,
        })? {
            QueueResponse::Metadata(value) => value.try_into(),
            _ => Err(Channel::unexpected()),
        }
    }
}
