use crate::Channel;
use anyq_daemon_api::{QueueRequest, QueueResponse};
use anyq_queue_api::{Result, SetTopicAutostart, Topic};

pub struct RemoteSetTopicAutostart {
    channel: Channel,
}

impl RemoteSetTopicAutostart {
    pub fn new(channel: Channel) -> Self {
        Self { channel }
    }
}

impl SetTopicAutostart for RemoteSetTopicAutostart {
    fn set_autostart(&mut self, name: &str, enabled: bool) -> Result<Topic> {
        match self.channel.request(QueueRequest::SetTopicAutostart {
            name: name.into(),
            enabled,
        })? {
            QueueResponse::Topic(value) => value.try_into(),
            _ => Err(Channel::unexpected()),
        }
    }
}
