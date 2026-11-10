use crate::{NativeChild, drain::Drain, native_entry::NativeEntry, prepare};
use anyq_queue_api::{ExecutionDirectory, Result};
use anyq_shell_api::{Command, Plan, PreparedPlan};
use anyq_shell_runtime::{ManagedProcess, PreparePlan, SpawnProcess};

pub struct NativeFactory {
    #[cfg(unix)]
    executable: std::path::PathBuf,
}

impl NativeFactory {
    pub fn new(_executable: std::path::PathBuf) -> Self {
        Self {
            #[cfg(unix)]
            executable: _executable,
        }
    }
}

impl PreparePlan for NativeFactory {
    fn prepare(&mut self, plan: Plan, origin: Option<ExecutionDirectory>) -> Result<PreparedPlan> {
        prepare::prepare(plan, origin)
    }
}

impl SpawnProcess for NativeFactory {
    fn spawn(
        &mut self,
        command: &Command,
    ) -> std::result::Result<Box<dyn ManagedProcess>, anyq_shell_runtime::LaunchError> {
        #[cfg(target_os = "linux")]
        rustix::process::set_child_subreaper(Some(rustix::process::Pid::INIT)).map_err(
            |error| {
                anyq_shell_runtime::LaunchError::NotStarted(format!(
                    "native child reaper is unavailable: {error}"
                ))
            },
        )?;
        #[cfg(unix)]
        let mut native = std::process::Command::new(&command.program);
        #[cfg(windows)]
        let mut native = windows_spawn::Command::new(&command.program);
        native
            .args(&command.arguments)
            .envs(&command.environment)
            .current_dir(
                prepare::directory(&command.directory).map_err(|error| {
                    anyq_shell_runtime::LaunchError::NotStarted(error.to_string())
                })?,
            );
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
        #[cfg(unix)]
        let mut child = NativeChild::spawn(&self.executable, &mut native)?;
        #[cfg(windows)]
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
