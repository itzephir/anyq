use crate::{Result, StartTaskOutcome};

pub trait StartTask {
    fn start(&mut self, name: &str) -> Result<StartTaskOutcome>;
}
