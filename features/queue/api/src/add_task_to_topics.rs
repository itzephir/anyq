use crate::{AddTaskToTopicsRequest, Result, Task};

pub trait AddTaskToTopics {
    fn add(&mut self, request: AddTaskToTopicsRequest) -> Result<Task>;
}
