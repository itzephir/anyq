use anyq_queue_api::{Error, Result};
use rustix::process::{Pid, WaitId, WaitIdOptions, kill_process_group, waitid};
use std::process::{Child, Command, ExitStatus};
use sysinfo::{ProcessRefreshKind, ProcessStatus, ProcessesToUpdate, System};

pub(crate) struct UnixChild {
    child: Child,
    pid: Pid,
    finished: bool,
    system: System,
}

impl UnixChild {
    pub fn spawn(command: &mut Command) -> Result<Self> {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
        let child = command
            .spawn()
            .map_err(|error| Error::Storage(format!("shell launch failed: {error}")))?;
        let pid = Pid::from_raw(child.id() as i32).expect("child PID");

        Ok(Self {
            child,
            pid,
            finished: false,
            system: System::new(),
        })
    }

    pub fn stdout(&mut self) -> std::process::ChildStdout {
        self.child.stdout.take().expect("piped stdout")
    }
    pub fn stderr(&mut self) -> std::process::ChildStderr {
        self.child.stderr.take().expect("piped stderr")
    }

    pub fn poll(&mut self) -> Result<Option<ExitStatus>> {
        let exited = waitid(
            WaitId::Pid(self.pid),
            WaitIdOptions::EXITED | WaitIdOptions::NOHANG | WaitIdOptions::NOWAIT,
        )
        .map_err(|error| Error::Storage(format!("cannot confirm shell leader: {error}")))?;
        if exited.is_none() {
            return Ok(None);
        }
        // Preserve the waitable leader PID until every member is stopped. No PGID
        // can be reused while that leader has not been reaped.
        self.system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing(),
        );
        // Query the owned child directly: macOS omits zombie leaders from
        // process inventory. NOWAIT confirms its identity remains reserved;
        // the complete live-member scan below independently checks descendants.
        let retained = waitid(
            WaitId::Pid(self.pid),
            WaitIdOptions::EXITED | WaitIdOptions::NOHANG | WaitIdOptions::NOWAIT,
        )
        .map_err(|error| Error::Storage(format!("cannot confirm retained leader: {error}")))?;
        if retained.is_none()
            || self
                .system
                .process(sysinfo::Pid::from_u32(std::process::id()))
                .is_none()
        {
            return Err(Error::Storage(
                "cannot confirm complete native process inventory".into(),
            ));
        }
        for (pid, process) in self.system.processes() {
            if matches!(
                process.status(),
                ProcessStatus::Zombie | ProcessStatus::Dead
            ) {
                continue;
            }
            let Some(pid) = Pid::from_raw(pid.as_u32() as i32) else {
                continue;
            };
            match rustix::process::getpgid(Some(pid)) {
                Ok(group) if group == self.pid => return Ok(None),
                Ok(_) | Err(rustix::io::Errno::SRCH) => {}
                Err(_) => {
                    return Err(Error::Storage(
                        "cannot confirm process-group membership".into(),
                    ));
                }
            }
        }
        let status = self
            .child
            .wait()
            .map_err(|error| Error::Storage(error.to_string()))?;
        self.finished = true;

        Ok(Some(status))
    }

    pub fn cancel(&mut self, force: bool) -> Result<()> {
        if self.finished {
            return Ok(());
        }
        match kill_process_group(
            self.pid,
            if force {
                rustix::process::Signal::KILL
            } else {
                rustix::process::Signal::TERM
            },
        ) {
            Ok(()) | Err(rustix::io::Errno::SRCH) => Ok(()),
            Err(error) => Err(Error::Storage(format!(
                "cannot terminate managed process group: {error}"
            ))),
        }
    }
}

impl Drop for UnixChild {
    fn drop(&mut self) {
        if !self.finished {
            let _ = self.cancel(true);
            // No persisted PID is used for cleanup or for a stopped certificate.
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        process::Stdio,
        time::{Duration, Instant},
    };

    #[test]
    fn waitable_exited_leader_is_retained_until_explicit_reap() {
        let mut command = Command::new("/bin/sh");
        command
            .args(["-c", "exit 0"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = UnixChild::spawn(&mut command).unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        while waitid(
            WaitId::Pid(child.pid),
            WaitIdOptions::EXITED | WaitIdOptions::NOHANG | WaitIdOptions::NOWAIT,
        )
        .unwrap()
        .is_none()
        {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(10));
        }

        assert!(
            waitid(
                WaitId::Pid(child.pid),
                WaitIdOptions::EXITED | WaitIdOptions::NOHANG | WaitIdOptions::NOWAIT
            )
            .unwrap()
            .is_some()
        );
        child.child.wait().unwrap();
        child.finished = true;
    }
}
