use crate::{RegisterTaskRequest, Result, Task};

pub trait RegisterTask {
    fn register(&mut self, request: RegisterTaskRequest) -> Result<Task>;
}
