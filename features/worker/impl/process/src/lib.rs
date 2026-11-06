//! POSIX process group adapter for macOS/Linux.
use anyq_queue_api::{Assignment, Error, Result};
use anyq_worker_execution::{Process, Runner, Wait};
use std::{
    io::Write,
    os::unix::process::{CommandExt, ExitStatusExt},
    process::{Child, Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

fn io(error: std::io::Error) -> Error {
    Error::Storage(error.to_string())
}
pub struct CommandRunner {
    command: Vec<String>,
}
impl CommandRunner {
    pub fn new(command: Vec<String>) -> Result<Self> {
        if command.first().is_none_or(|s| s.is_empty()) {
            return Err(Error::Invalid("worker command is required after --".into()));
        }
        Ok(Self { command })
    }
}
struct ChildProcess {
    child: Child,
    done: bool,
}
impl ChildProcess {
    #[allow(unsafe_code)]
    fn kill_group(&self) -> Result<()> {
        // SAFETY: child PID is positive, created in its own process group; a negative
        // PID targets precisely that group. ESRCH means it has already exited.
        let result = unsafe { libc::kill(-(self.child.id() as i32), libc::SIGKILL) };
        if result != 0 {
            let error = std::io::Error::last_os_error();
            if error.raw_os_error() != Some(libc::ESRCH) {
                return Err(io(error));
            }
        }
        Ok(())
    }
}
impl Runner for CommandRunner {
    fn start(&mut self, a: &Assignment) -> Result<Box<dyn Process>> {
        let mut cmd = Command::new(&self.command[0]);
        cmd.args(&self.command[1..])
            .process_group(0)
            .stdin(Stdio::piped())
            .stdout(Stdio::from(std::io::stderr()))
            .stderr(Stdio::inherit())
            .env("ANYQ_MESSAGE_ID", a.message.id.to_string())
            .env("ANYQ_TOPIC", &a.message.topic)
            .env("ANYQ_GROUP", &a.delivery.group)
            .env(
                "ANYQ_CONSUMER",
                a.delivery.consumer.as_deref().unwrap_or_default(),
            )
            .env("ANYQ_ATTEMPT", a.delivery.attempts.to_string());
        let mut child = cmd.spawn().map_err(io)?;
        let stdin = child.stdin.take();
        let payload = a.message.payload.clone();
        // A command may never read stdin; pipe backpressure must not block heartbeats.
        thread::spawn(move || {
            if let Some(mut pipe) = stdin {
                let _ = pipe.write_all(payload.as_bytes());
            }
        });
        Ok(Box::new(ChildProcess { child, done: false }))
    }
}
impl Process for ChildProcess {
    fn poll(&mut self) -> Result<Option<i32>> {
        match self.child.try_wait().map_err(io)? {
            Some(status) => {
                // A finished parent must not leave resource-using descendants behind.
                self.kill_group()?;
                self.done = true;
                Ok(Some(
                    status.code().unwrap_or(128 + status.signal().unwrap_or(1)),
                ))
            }
            None => Ok(None),
        }
    }
    fn stop(&mut self) -> Result<()> {
        if !self.done {
            self.kill_group()?;
            self.child.wait().map_err(io)?;
            self.done = true;
        }
        Ok(())
    }
}
impl Drop for ChildProcess {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

pub struct SystemWait {
    cancelled: Arc<AtomicBool>,
    registrations: Vec<signal_hook::SigId>,
}
impl SystemWait {
    pub fn new() -> Result<Self> {
        let cancelled = Arc::new(AtomicBool::new(false));
        let mut registrations = Vec::new();
        for signal in [signal_hook::consts::SIGINT, signal_hook::consts::SIGTERM] {
            match signal_hook::flag::register(signal, cancelled.clone()) {
                Ok(id) => registrations.push(id),
                Err(error) => {
                    for id in registrations {
                        signal_hook::low_level::unregister(id);
                    }
                    return Err(io(error));
                }
            }
        }
        Ok(Self {
            cancelled,
            registrations,
        })
    }
}
impl Wait for SystemWait {
    fn wait_ms(&mut self, duration: u64) {
        let end = Instant::now() + Duration::from_millis(duration);
        while !self.stopped() && Instant::now() < end {
            thread::sleep(
                end.saturating_duration_since(Instant::now())
                    .min(Duration::from_millis(50)),
            );
        }
    }
    fn stopped(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }
}
impl Drop for SystemWait {
    fn drop(&mut self) {
        for id in self.registrations.drain(..) {
            signal_hook::low_level::unregister(id);
        }
    }
}
