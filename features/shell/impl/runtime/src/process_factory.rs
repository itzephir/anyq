use crate::ManagedProcess;
use anyq_queue_api::{ExecutionDirectory, Result};
use anyq_shell_api::{Command, Plan, PreparedPlan};

pub trait ProcessFactory {
    fn prepare(&mut self, plan: Plan, origin: Option<ExecutionDirectory>) -> Result<PreparedPlan>;
    fn spawn(&mut self, command: &Command) -> Result<Box<dyn ManagedProcess>>;
}
