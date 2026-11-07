use crate::{Error, Result, validate_name};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Topic {
    pub name: String,
    pub capacity: u32,
}

impl Topic {
    pub fn new(name: String, capacity: u32) -> Result<Self> {
        validate_name(&name)?;

        if capacity == 0 {
            return Err(Error::Invalid("capacity must be positive".into()));
        }

        Ok(Self { name, capacity })
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
