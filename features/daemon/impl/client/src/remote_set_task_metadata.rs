use crate::Channel;
use anyq_daemon_api::{QueueRequest, QueueResponse};
use anyq_queue_api::{Metadata, Result, SetTaskMetadata, SetTaskMetadataRequest};

pub struct RemoteSetTaskMetadata {
    channel: Channel,
}

impl RemoteSetTaskMetadata {
    pub fn new(channel: Channel) -> Self {
        Self { channel }
    }
}

impl SetTaskMetadata for RemoteSetTaskMetadata {
    fn set(&mut self, request: SetTaskMetadataRequest) -> Result<Metadata> {
        match self.channel.request(QueueRequest::SetTaskMetadata {
            task: request.task,
            key: request.key,
            value: request.value.into(),
        })? {
            QueueResponse::Metadata(value) => value.try_into(),
            _ => Err(Channel::unexpected()),
        }
    }
}
