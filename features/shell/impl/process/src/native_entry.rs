use crate::{NativeChild, drain::Drain};
use anyq_queue_api::Result;
use anyq_shell_api::{EntryOutcome, EntryOutput};
use anyq_shell_runtime::ManagedProcess;
use std::{
    process::ExitStatus,
    time::{Duration, Instant},
};

pub(crate) struct NativeEntry {
    pub child: NativeChild,
    pub stdout: Drain,
    pub stderr: Drain,
    pub exit: Option<ExitStatus>,
    pub cancelling: Option<Instant>,
}

impl ManagedProcess for NativeEntry {
    fn poll(&mut self) -> Result<Option<EntryOutcome>> {
        if self
            .cancelling
            .is_some_and(|at| at.elapsed() >= Duration::from_millis(300))
        {
            self.child.cancel(true)?;
        }
        if self.exit.is_none() {
            self.exit = self.child.poll()?;
        }
        if let Some(status) = self.exit {
            if self.stdout.done() && self.stderr.done() {
                return Ok(Some(EntryOutcome {
                    code: status.code(),
                    success: status.success(),
                    output: self.output(),
                }));
            }
        }

        Ok(None)
    }

    fn cancel(&mut self) -> Result<()> {
        if self.cancelling.is_none() {
            self.cancelling = Some(Instant::now());
        }
        self.child.cancel(false)
    }

    fn output(&self) -> EntryOutput {
        let (stdout, stdout_truncated) = self.stdout.snapshot();
        let (stderr, stderr_truncated) = self.stderr.snapshot();

        EntryOutput {
            stdout,
            stderr,
            stdout_truncated,
            stderr_truncated,
        }
    }
}
