use anyq_queue_api::{Error, Result};
use anyq_shell_runtime::LaunchError;
use std::{
    os::windows::io::{AsHandle, AsRawHandle},
    process::ExitStatus,
};
use windows_spawn::{Child, Command, Job, SpawnOptions};

pub(crate) struct WindowsChild {
    child: Child,
    job: win32job::Job,
    kill_job: Job,
    exit: Option<ExitStatus>,
}

impl WindowsChild {
    pub fn spawn(command: &mut Command) -> std::result::Result<Self, LaunchError> {
        let kill_job = Job::create().map_err(|error| LaunchError::NotStarted(error.to_string()))?;
        kill_job
            .set_kill_on_close(true)
            .map_err(|error| LaunchError::NotStarted(error.to_string()))?;
        let job =
            win32job::Job::create().map_err(|error| LaunchError::NotStarted(error.to_string()))?;
        // Membership in the private kill job is established at creation. The
        // independent query job is also assigned before any user code resumes.
        let suspended = command
            .spawn_suspended_with(SpawnOptions::new().job(&kill_job))
            .map_err(|error| LaunchError::NotStarted(format!("shell launch failed: {error}")))?;
        job.assign_process(suspended.as_handle().as_raw_handle() as isize)
            .map_err(|error| LaunchError::NotStarted(error.to_string()))?;
        let child = suspended
            .resume()
            .map_err(|error| LaunchError::Uncertain(error.to_string()))?;

        Ok(Self {
            child,
            job,
            kill_job,
            exit: None,
        })
    }

    pub fn stdout(&mut self) -> windows_spawn::ChildStdout {
        self.child.stdout.take().expect("piped stdout")
    }
    pub fn stderr(&mut self) -> windows_spawn::ChildStderr {
        self.child.stderr.take().expect("piped stderr")
    }

    pub fn poll(&mut self) -> Result<Option<ExitStatus>> {
        if self.exit.is_none() {
            self.exit = self
                .child
                .try_wait()
                .map_err(|error| Error::Storage(error.to_string()))?;
        }
        // An empty query alone is insufficient: retain and verify the original
        // root handle, including after requesting termination of the whole job.
        let Some(exit) = self.exit else {
            return Ok(None);
        };
        let pids = self.job.query_process_id_list().map_err(|error| {
            Error::Storage(format!("cannot confirm native job termination: {error}"))
        })?;
        if !pids.is_empty() {
            return Ok(None);
        }

        Ok(Some(exit))
    }

    pub fn cancel(&mut self, _force: bool) -> Result<()> {
        self.kill_job
            .terminate(1)
            .map_err(|error| Error::Storage(format!("cannot terminate native job: {error}")))
    }
}
