use crate::{LaunchError, ManagedProcess, SpawnProcess};
use anyq_shell_api::Command;

pub(crate) struct LaunchFailure {
    pub uncertain: bool,
}

impl SpawnProcess for LaunchFailure {
    fn spawn(
        &mut self,
        _command: &Command,
    ) -> std::result::Result<Box<dyn ManagedProcess>, LaunchError> {
        Err(if self.uncertain {
            LaunchError::Uncertain("unknown launch result".into())
        } else {
            LaunchError::NotStarted("no process started".into())
        })
    }
}
