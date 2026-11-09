use crate::{Result, Task};

pub trait AddTaskToTopic {
    fn add(&mut self, task: &str, topic: &str) -> Result<Task>;
}
