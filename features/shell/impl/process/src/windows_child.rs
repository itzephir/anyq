use anyq_queue_api::{Error, Result};
use std::{
    os::windows::io::{AsHandle, AsRawHandle},
    process::ExitStatus,
};
use windows_spawn::{Child, Command, DropPolicy, SpawnOptions};

pub(crate) struct WindowsChild {
    child: Option<Child>,
    job: win32job::Job,
    exit: Option<ExitStatus>,
}

impl WindowsChild {
    pub fn spawn(command: &mut Command) -> Result<Self> {
        let job = win32job::Job::create().map_err(|error| Error::Storage(error.to_string()))?;
        // The SDK creates its kill-on-close job at process creation. This second
        // query job is assigned before the suspended process can run or fork.
        let suspended = command
            .spawn_suspended_with(SpawnOptions::new().drop_policy(DropPolicy::KillTree))
            .map_err(|error| Error::Storage(format!("shell launch failed: {error}")))?;
        job.assign_process(suspended.as_handle().as_raw_handle() as isize)
            .map_err(|error| Error::Storage(error.to_string()))?;
        let child = suspended
            .resume()
            .map_err(|error| Error::Storage(error.to_string()))?;

        Ok(Self {
            child: Some(child),
            job,
            exit: None,
        })
    }

    pub fn stdout(&mut self) -> windows_spawn::ChildStdout {
        self.child
            .as_mut()
            .unwrap()
            .stdout
            .take()
            .expect("piped stdout")
    }
    pub fn stderr(&mut self) -> windows_spawn::ChildStderr {
        self.child
            .as_mut()
            .unwrap()
            .stderr
            .take()
            .expect("piped stderr")
    }

    pub fn poll(&mut self) -> Result<Option<ExitStatus>> {
        if let Some(child) = &mut self.child {
            if self.exit.is_none() {
                self.exit = child
                    .try_wait()
                    .map_err(|error| Error::Storage(error.to_string()))?;
            }
        }
        let pids = self.job.query_process_id_list().map_err(|error| {
            Error::Storage(format!("cannot confirm native job termination: {error}"))
        })?;
        if !pids.is_empty() {
            return Ok(None);
        }
        if let Some(exit) = self.exit {
            return Ok(Some(exit));
        }
        // Cancellation dropped the SDK child and its kill-on-close job. The
        // independently held query job is now empty, proving all members stopped.
        use std::os::windows::process::ExitStatusExt;
        Ok(Some(ExitStatus::from_raw(1)))
    }

    pub fn cancel(&mut self, _force: bool) -> Result<()> {
        drop(self.child.take());
        Ok(())
    }
}
