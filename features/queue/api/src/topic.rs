use crate::{Error, Result, parse_topic_name};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Topic {
    pub name: String,
    pub capacity: u32,
    pub autostart: Option<bool>,
}

impl Topic {
    pub const DEFAULT_CAPACITY: u32 = 1;

    pub fn new(name: String, capacity: u32) -> Result<Self> {
        Self::restore(parse_topic_name(&name)?, capacity)
    }

    /// Reconstruct a canonical stored name without interpreting backticks again.
    pub fn restore(name: String, capacity: u32) -> Result<Self> {
        if capacity == 0 {
            return Err(Error::Invalid("capacity must be positive".into()));
        }

        Ok(Self {
            name,
            capacity,
            autostart: None,
        })
    }

    pub fn require_capacity_for(&self, running: u64) -> Result<()> {
        if running > u64::from(self.capacity) {
            return Err(Error::Conflict(
                "capacity cannot be lower than the number of running tasks".into(),
            ));
        }

        Ok(())
    }
}
