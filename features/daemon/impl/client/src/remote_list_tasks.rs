use crate::Channel;
use anyq_daemon_api::{QueueRequest, QueueResponse};
use anyq_queue_api::{ListTasks, Result};

pub struct RemoteListTasks {
    channel: Channel,
}

impl RemoteListTasks {
    pub fn new(channel: Channel) -> Self {
        Self { channel }
    }
}

impl ListTasks for RemoteListTasks {
    fn list(&mut self) -> Result<Vec<String>> {
        match self.channel.request(QueueRequest::ListTasks)? {
            QueueResponse::Names(values) => Ok(values),
            _ => Err(Channel::unexpected()),
        }
    }
}
