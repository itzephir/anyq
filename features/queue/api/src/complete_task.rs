use crate::{Result, Task};

pub trait CompleteTask {
    fn complete(&mut self, name: &str, force: bool) -> Result<Task>;
}
