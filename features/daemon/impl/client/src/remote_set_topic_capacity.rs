use crate::Channel;
use anyq_daemon_api::{QueueRequest, QueueResponse};
use anyq_queue_api::{Result, SetTopicCapacity, Topic};

pub struct RemoteSetTopicCapacity {
    channel: Channel,
}

impl RemoteSetTopicCapacity {
    pub fn new(channel: Channel) -> Self {
        Self { channel }
    }
}

impl SetTopicCapacity for RemoteSetTopicCapacity {
    fn set_capacity(&mut self, topic: Topic) -> Result<Topic> {
        match self.channel.request(QueueRequest::SetTopicCapacity {
            topic: topic.into(),
        })? {
            QueueResponse::Topic(value) => value.try_into(),
            _ => Err(Channel::unexpected()),
        }
    }
}
