use crate::Channel;
use anyq_daemon_api::{QueueRequest, QueueResponse};
use anyq_queue_api::{AddTasksToTopic, AddTasksToTopicRequest, Result, Task};

pub struct RemoteAddTasksToTopic {
    channel: Channel,
}

impl RemoteAddTasksToTopic {
    pub fn new(channel: Channel) -> Self {
        Self { channel }
    }
}

impl AddTasksToTopic for RemoteAddTasksToTopic {
    fn add(&mut self, request: AddTasksToTopicRequest) -> Result<Vec<Task>> {
        match self.channel.request(QueueRequest::AddTasksToTopic {
            tasks: request.tasks,
            topic: request.topic,
        })? {
            QueueResponse::Tasks(values) => values.into_iter().map(TryInto::try_into).collect(),
            _ => Err(Channel::unexpected()),
        }
    }
}
