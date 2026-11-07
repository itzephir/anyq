use crate::{Result, Task};

pub trait RegisterTask {
    fn register(&mut self, name: String) -> Result<Task>;
}
