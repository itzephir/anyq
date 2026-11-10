use crate::Channel;
use anyq_daemon_api::{QueueRequest, QueueResponse};
use anyq_queue_api::{ListTopics, Result, Topic};

pub struct RemoteListTopics {
    channel: Channel,
}

impl RemoteListTopics {
    pub fn new(channel: Channel) -> Self {
        Self { channel }
    }
}

impl ListTopics for RemoteListTopics {
    fn list(&mut self) -> Result<Vec<Topic>> {
        match self.channel.request(QueueRequest::ListTopics)? {
            QueueResponse::Topics(values) => values.into_iter().map(TryInto::try_into).collect(),
            _ => Err(Channel::unexpected()),
        }
    }
}
