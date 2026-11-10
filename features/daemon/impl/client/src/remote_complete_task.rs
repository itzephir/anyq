use crate::Channel;
use anyq_daemon_api::{QueueRequest, QueueResponse};
use anyq_queue_api::{CompleteTask, Result, Task};

pub struct RemoteCompleteTask {
    channel: Channel,
}

impl RemoteCompleteTask {
    pub fn new(channel: Channel) -> Self {
        Self { channel }
    }
}

impl CompleteTask for RemoteCompleteTask {
    fn complete(&mut self, name: &str, force: bool) -> Result<Task> {
        match self.channel.request(QueueRequest::CompleteTask {
            name: name.into(),
            force,
        })? {
            QueueResponse::Task(value) => value.try_into(),
            _ => Err(Channel::unexpected()),
        }
    }
}
