use crate::{Result, StartOutcome};

pub trait TryStartTask {
    fn try_start(&mut self, name: &str) -> Result<StartOutcome>;
}
