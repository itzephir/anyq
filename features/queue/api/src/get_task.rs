use crate::{Result, Task};

pub trait GetTask {
    fn get(&mut self, name: &str) -> Result<Task>;
}
