use anyq_queue_api::{Error, Result};
use anyq_shell_runtime::LaunchError;
use rustix::process::{
    Pid, WaitId, WaitIdOptions, kill_process_group, test_kill_process_group, waitid,
};
use std::{
    io::{Read, Write},
    path::Path,
    process::{Child, Command, ExitStatus, Stdio},
};

pub(crate) struct UnixChild {
    child: Child,
    anchor: Child,
    pid: Pid,
    exit: Option<ExitStatus>,
    finished: bool,
    uncertain: bool,
}

impl UnixChild {
    pub fn spawn(
        executable: &Path,
        command: &mut Command,
    ) -> std::result::Result<Self, LaunchError> {
        use std::os::unix::process::CommandExt;
        let mut anchor = Command::new(executable)
            .args(["plugin-runtime", "shell-anchor"])
            .process_group(0)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| {
                LaunchError::NotStarted(format!("shell anchor launch failed: {error}"))
            })?;
        let pid = Pid::from_raw(anchor.id() as i32).expect("anchor PID");
        let mut ready = [0_u8];
        if anchor
            .stdout
            .as_mut()
            .expect("anchor stdout")
            .read_exact(&mut ready)
            .is_err()
            || ready != *b"R"
        {
            let _ = anchor.kill();
            let _ = anchor.wait();
            return Err(LaunchError::NotStarted(
                "shell anchor did not initialize".into(),
            ));
        }
        command.process_group(i32::try_from(anchor.id()).expect("native anchor PID"));
        let child = match command.spawn() {
            Ok(child) => child,
            Err(error) => {
                let _ = anchor.stdin.as_mut().expect("anchor stdin").write_all(b"Q");
                let _ = anchor.wait();
                return Err(LaunchError::NotStarted(format!(
                    "shell launch failed: {error}"
                )));
            }
        };
        let handoff = anchor
            .stdin
            .as_mut()
            .expect("anchor stdin")
            .write_all(b"L")
            .and_then(|_| {
                anchor
                    .stdout
                    .as_mut()
                    .expect("anchor stdout")
                    .read_exact(&mut ready)
            });
        // Once a task process exists, a handshake error is uncertainty, never
        // a no-process failure that could authorize resource release.
        let uncertain = handoff.is_err() || ready != *b"R";

        Ok(Self {
            child,
            anchor,
            pid,
            exit: None,
            finished: false,
            uncertain,
        })
    }

    pub fn stdout(&mut self) -> std::process::ChildStdout {
        self.child.stdout.take().expect("piped stdout")
    }
    pub fn stderr(&mut self) -> std::process::ChildStderr {
        self.child.stderr.take().expect("piped stderr")
    }

    pub fn poll(&mut self) -> Result<Option<ExitStatus>> {
        if self.uncertain {
            return Err(Error::Storage("shell anchor handoff is uncertain".into()));
        }
        if waitid(
            WaitId::Pid(self.pid),
            WaitIdOptions::EXITED | WaitIdOptions::NOHANG | WaitIdOptions::NOWAIT,
        )
        .map_err(|error| Error::Storage(error.to_string()))?
        .is_some()
        {
            return Err(Error::Storage(
                "shell anchor exited before group confirmation".into(),
            ));
        }
        if self.exit.is_none() {
            self.exit = self
                .child
                .try_wait()
                .map_err(|error| Error::Storage(error.to_string()))?;
        }
        let Some(exit) = self.exit else {
            return Ok(None);
        };
        #[cfg(target_os = "linux")]
        loop {
            // Adopted descendants belong to this isolated plugin process. Reap
            // only this action's group, never another action or the live anchor.
            match rustix::process::waitpgid(self.pid, rustix::process::WaitOptions::NOHANG) {
                Ok(Some(_)) => continue,
                Ok(None) | Err(rustix::io::Errno::CHILD) => break,
                Err(error) => {
                    return Err(Error::Storage(format!(
                        "cannot reap managed descendants: {error}"
                    )));
                }
            }
        }
        match test_kill_process_group(self.pid) {
            Ok(()) => return Ok(None),
            // Darwin can return EPERM while the last terminated member is
            // being reparented/reaped. Keep ownership and retry; only ESRCH
            // certifies an empty group. Persistent denial stays unresolved.
            Err(rustix::io::Errno::PERM) => return Ok(None),
            Err(rustix::io::Errno::SRCH) => {}
            Err(error) => {
                return Err(Error::Storage(format!(
                    "cannot confirm native group termination: {error}"
                )));
            }
        }
        // The anchor is alive outside the target group: its PID prevents group
        // reuse while the kernel's ESRCH certifies there are no members at all.
        self.anchor
            .stdin
            .as_mut()
            .expect("anchor stdin")
            .write_all(b"Q")
            .map_err(|error| Error::Storage(error.to_string()))?;
        self.anchor
            .wait()
            .map_err(|error| Error::Storage(error.to_string()))?;
        self.finished = true;

        Ok(Some(exit))
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
            Ok(()) | Err(rustix::io::Errno::SRCH | rustix::io::Errno::PERM) => Ok(()),
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
            drop(self.anchor.stdin.take());
            let _ = self.anchor.wait();
            let _ = self.child.try_wait();
        }
    }
}
