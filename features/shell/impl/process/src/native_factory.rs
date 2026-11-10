use crate::{NativeChild, drain::Drain, native_entry::NativeEntry, prepare};
use anyq_queue_api::{ExecutionDirectory, Result};
use anyq_shell_api::{Command, Plan, PreparedPlan};
use anyq_shell_runtime::{ManagedProcess, ProcessFactory};

pub struct NativeFactory;

impl ProcessFactory for NativeFactory {
    fn prepare(&mut self, plan: Plan, origin: Option<ExecutionDirectory>) -> Result<PreparedPlan> {
        prepare::prepare(plan, origin)
    }

    fn spawn(&mut self, command: &Command) -> Result<Box<dyn ManagedProcess>> {
        #[cfg(unix)]
        let mut native = std::process::Command::new(&command.program);
        #[cfg(windows)]
        let mut native = windows_spawn::Command::new(&command.program);
        native
            .args(&command.arguments)
            .envs(&command.environment)
            .current_dir(prepare::directory(&command.directory)?);
        #[cfg(unix)]
        native
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        #[cfg(windows)]
        {
            native
                .stdin(windows_spawn::Stdio::null())
                .stdout(windows_spawn::Stdio::piped())
                .stderr(windows_spawn::Stdio::piped());
            if let Some(text) = &command.raw_shell {
                native.raw_arg(text);
            }
        }
        let mut child = NativeChild::spawn(&mut native)?;
        let stdout = Drain::start(child.stdout());
        let stderr = Drain::start(child.stderr());

        Ok(Box::new(NativeEntry {
            child,
            stdout,
            stderr,
            exit: None,
            cancelling: None,
        }))
    }
}
