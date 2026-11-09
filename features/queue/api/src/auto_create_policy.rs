use crate::{Error, Result, format_name};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AutoCreatePolicy {
    pub tasks: bool,
    pub topics: bool,
}

impl AutoCreatePolicy {
    pub fn require_task_creation(&self, name: &str) -> Result<()> {
        if !self.tasks {
            return Err(Error::NotFound(format!(
                "task '{}' does not exist",
                format_name(name)
            )));
        }

        Ok(())
    }

    pub fn require_topic_creation(&self, name: &str) -> Result<()> {
        if !self.topics {
            return Err(Error::NotFound(format!(
                "topic '{}' does not exist",
                format_name(name)
            )));
        }

        Ok(())
    }
}
