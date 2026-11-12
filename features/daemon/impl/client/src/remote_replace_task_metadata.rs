use crate::Channel;
use anyq_daemon_api::{QueueRequest, QueueResponse};
use anyq_queue_api::{Metadata, ReplaceTaskMetadata, ReplaceTaskMetadataRequest, Result};

pub struct RemoteReplaceTaskMetadata {
    channel: Channel,
}

impl RemoteReplaceTaskMetadata {
    pub fn new(channel: Channel) -> Self {
        Self { channel }
    }
}

impl ReplaceTaskMetadata for RemoteReplaceTaskMetadata {
    fn replace(&mut self, request: ReplaceTaskMetadataRequest) -> Result<Metadata> {
        match self.channel.request(QueueRequest::ReplaceTaskMetadata {
            task: request.task,
            metadata: request.metadata.into(),
        })? {
            QueueResponse::Metadata(value) => value.try_into(),
            _ => Err(Channel::unexpected()),
        }
    }
}
