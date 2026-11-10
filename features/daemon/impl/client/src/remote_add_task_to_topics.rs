use crate::Channel;
use anyq_daemon_api::{QueueRequest, QueueResponse};
use anyq_queue_api::{AddTaskToTopics, AddTaskToTopicsRequest, Result, Task};

pub struct RemoteAddTaskToTopics {
    channel: Channel,
}

impl RemoteAddTaskToTopics {
    pub fn new(channel: Channel) -> Self {
        Self { channel }
    }
}

impl AddTaskToTopics for RemoteAddTaskToTopics {
    fn add(&mut self, request: AddTaskToTopicsRequest) -> Result<Task> {
        match self.channel.request(QueueRequest::AddTaskToTopics {
            task: request.task,
            topics: request.topics,
        })? {
            QueueResponse::Task(value) => value.try_into(),
            _ => Err(Channel::unexpected()),
        }
    }
}
