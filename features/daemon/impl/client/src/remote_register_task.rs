use crate::Channel;
use anyq_daemon_api::{QueueRequest, QueueResponse};
use anyq_queue_api::{RegisterTask, RegisterTaskRequest, Result, Task};

pub struct RemoteRegisterTask {
    channel: Channel,
}

impl RemoteRegisterTask {
    pub fn new(channel: Channel) -> Self {
        Self { channel }
    }
}

impl RegisterTask for RemoteRegisterTask {
    fn register(&mut self, request: RegisterTaskRequest) -> Result<Task> {
        match self.channel.request(QueueRequest::RegisterTask {
            name: request.name,
            topics: request.topics,
            start: request.start.into(),
            metadata: request.metadata.into(),
        })? {
            QueueResponse::Task(value) => value.try_into(),
            _ => Err(Channel::unexpected()),
        }
    }
}
