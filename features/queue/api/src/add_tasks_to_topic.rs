use crate::{AddTasksToTopicRequest, Result, Task};

pub trait AddTasksToTopic {
    fn add(&mut self, request: AddTasksToTopicRequest) -> Result<Vec<Task>>;
}
