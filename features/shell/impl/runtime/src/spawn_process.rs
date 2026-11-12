use crate::{LaunchError, ManagedProcess};
use anyq_shell_api::Command;

pub trait SpawnProcess {
    fn spawn(
        &mut self,
        command: &Command,
    ) -> std::result::Result<Box<dyn ManagedProcess>, LaunchError>;
}
