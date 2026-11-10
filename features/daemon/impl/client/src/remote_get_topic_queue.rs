use crate::Channel;
use anyq_daemon_api::{QueueRequest, QueueResponse};
use anyq_queue_api::{GetTopicQueue, Result, TopicQueue};

pub struct RemoteGetTopicQueue {
    channel: Channel,
}

impl RemoteGetTopicQueue {
    pub fn new(channel: Channel) -> Self {
        Self { channel }
    }
}

impl GetTopicQueue for RemoteGetTopicQueue {
    fn get(&mut self, name: &str) -> Result<TopicQueue> {
        match self
            .channel
            .request(QueueRequest::GetTopicQueue { name: name.into() })?
        {
            QueueResponse::TopicQueue(value) => value.try_into(),
            _ => Err(Channel::unexpected()),
        }
    }
}
