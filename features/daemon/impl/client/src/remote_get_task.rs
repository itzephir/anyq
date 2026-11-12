use crate::Channel;
use anyq_daemon_api::{QueueRequest, QueueResponse};
use anyq_queue_api::{GetTask, Result, Task};

pub struct RemoteGetTask {
    channel: Channel,
}

impl RemoteGetTask {
    pub fn new(channel: Channel) -> Self {
        Self { channel }
    }
}

impl GetTask for RemoteGetTask {
    fn get(&mut self, name: &str) -> Result<Task> {
        match self
            .channel
            .request(QueueRequest::GetTask { name: name.into() })?
        {
            QueueResponse::Task(value) => value.try_into(),
            _ => Err(Channel::unexpected()),
        }
    }
}
