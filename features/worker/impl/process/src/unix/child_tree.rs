use std::{
    io,
    os::unix::process::{CommandExt, ExitStatusExt},
    process::{Child, Command},
};

pub struct ChildTree {
    pub child: Child,
    done: bool,
}
impl ChildTree {
    pub fn spawn(command: &mut Command) -> io::Result<Self> {
        Ok(Self {
            child: command.process_group(0).spawn()?,
            done: false,
        })
    }
    #[allow(unsafe_code)]
    fn kill_group(&self) -> io::Result<()> {
        // SAFETY: this child was created as leader of its own process group.
        let result = unsafe { libc::kill(-(self.child.id() as i32), libc::SIGKILL) };
        if result != 0 {
            let error = io::Error::last_os_error();
            if error.raw_os_error() != Some(libc::ESRCH) {
                return Err(error);
            }
        }
        Ok(())
    }
    pub fn poll(&mut self) -> io::Result<Option<i32>> {
        match self.child.try_wait()? {
            Some(status) => {
                self.kill_group()?;
                self.done = true;
                Ok(Some(
                    status.code().unwrap_or(128 + status.signal().unwrap_or(1)),
                ))
            }
            None => Ok(None),
        }
    }
    pub fn stop(&mut self) -> io::Result<()> {
        if !self.done {
            self.kill_group()?;
            self.child.wait()?;
            self.done = true;
        }
        Ok(())
    }
}
impl Drop for ChildTree {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
