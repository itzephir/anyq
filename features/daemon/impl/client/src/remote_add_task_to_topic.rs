use crate::Channel;
use anyq_daemon_api::{QueueRequest, QueueResponse};
use anyq_queue_api::{AddTaskToTopic, Result, Task};

pub struct RemoteAddTaskToTopic {
    channel: Channel,
}

impl RemoteAddTaskToTopic {
    pub fn new(channel: Channel) -> Self {
        Self { channel }
    }
}

impl AddTaskToTopic for RemoteAddTaskToTopic {
    fn add(&mut self, task: &str, topic: &str) -> Result<Task> {
        match self.channel.request(QueueRequest::AddTaskToTopic {
            task: task.into(),
            topic: topic.into(),
        })? {
            QueueResponse::Task(value) => value.try_into(),
            _ => Err(Channel::unexpected()),
        }
    }
}
