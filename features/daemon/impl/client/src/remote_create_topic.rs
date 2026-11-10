use crate::Channel;
use anyq_daemon_api::{QueueRequest, QueueResponse};
use anyq_queue_api::{CreateTopic, Result, Topic};

pub struct RemoteCreateTopic {
    channel: Channel,
}

impl RemoteCreateTopic {
    pub fn new(channel: Channel) -> Self {
        Self { channel }
    }
}

impl CreateTopic for RemoteCreateTopic {
    fn create(&mut self, topic: Topic) -> Result<Topic> {
        match self.channel.request(QueueRequest::CreateTopic {
            topic: topic.into(),
        })? {
            QueueResponse::Topic(value) => value.try_into(),
            _ => Err(Channel::unexpected()),
        }
    }
}
