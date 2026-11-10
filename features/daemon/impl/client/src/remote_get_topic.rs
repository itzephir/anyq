use crate::Channel;
use anyq_daemon_api::{QueueRequest, QueueResponse};
use anyq_queue_api::{GetTopic, Result, Topic};

pub struct RemoteGetTopic {
    channel: Channel,
}

impl RemoteGetTopic {
    pub fn new(channel: Channel) -> Self {
        Self { channel }
    }
}

impl GetTopic for RemoteGetTopic {
    fn get(&mut self, name: &str) -> Result<Topic> {
        match self
            .channel
            .request(QueueRequest::GetTopic { name: name.into() })?
        {
            QueueResponse::Topic(value) => value.try_into(),
            _ => Err(Channel::unexpected()),
        }
    }
}
