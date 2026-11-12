use crate::{CompleteTaskOnceRequest, Result, Task};

pub trait CompleteTaskOnce {
    fn complete(&mut self, request: CompleteTaskOnceRequest) -> Result<Task>;
}
