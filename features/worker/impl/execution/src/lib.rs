//! Command execution policy; no OS/process/time IO.
#[cfg(test)]
mod tests;
use anyq_queue_api::*;
use anyq_worker_api::{Run, Settings, Worker};

pub trait Process {
    /// None means running; Some contains the exit code (signals are nonzero).
    fn poll(&mut self) -> Result<Option<i32>>;
    fn stop(&mut self) -> Result<()>;
}
pub trait Runner {
    fn start(&mut self, assignment: &Assignment) -> Result<Box<dyn Process>>;
}
pub trait Wait {
    fn wait_ms(&mut self, duration: u64);
    fn stopped(&self) -> bool;
}

pub struct Execution<R, W> {
    queue: Box<dyn Queue>,
    runner: R,
    wait: W,
    settings: Settings,
}
impl<R, W> Execution<R, W> {
    pub fn new(queue: Box<dyn Queue>, runner: R, wait: W, settings: Settings) -> Result<Self> {
        settings.validate()?;
        Ok(Self {
            queue,
            runner,
            wait,
            settings,
        })
    }
}
impl<R: Runner, W: Wait> Worker for Execution<R, W> {
    fn run_one(&mut self) -> Result<Run> {
        if self.wait.stopped() {
            return Ok(Run::Stopped);
        }
        // Validate next destination before running a potentially expensive external action.
        if let Some(next) = &self.settings.next {
            self.queue.execute(Command::Groups {
                topic: next.topic.clone(),
            })?;
        }
        let Output::Claimed(assignment) = self
            .queue
            .execute(Command::Claim(self.settings.claim.clone()))?
        else {
            return Err(Error::Conflict(
                "queue returned an unexpected claim result".into(),
            ));
        };
        let Some(assignment) = assignment else {
            return Ok(Run::Idle);
        };
        let lease = assignment.lease();
        let mut process = match self.runner.start(&assignment) {
            Ok(p) => p,
            Err(error) => {
                self.queue.execute(Command::Nack {
                    lease,
                    delay_ms: self.settings.retry_delay_ms,
                    reason: format!("command start failed: {error}"),
                })?;
                return Err(error);
            }
        };
        loop {
            if self.wait.stopped() {
                process.stop()?;
                self.queue.execute(Command::Nack {
                    lease,
                    delay_ms: self.settings.retry_delay_ms,
                    reason: "worker interrupted".into(),
                })?;
                return Ok(Run::Stopped);
            }
            match process.poll() {
                Ok(Some(code)) => {
                    if code == 0 {
                        self.queue.execute(Command::Ack {
                            lease,
                            result: Some("{\"exit_code\":0}".into()),
                            next: self.settings.next.clone(),
                        })?;
                        return Ok(Run::Succeeded(assignment.message.id));
                    }
                    self.queue.execute(Command::Nack {
                        lease,
                        delay_ms: self.settings.retry_delay_ms,
                        reason: format!("command exited with code {code}"),
                    })?;
                    return Ok(Run::Failed(assignment.message.id));
                }
                Ok(None) => {}
                Err(error) => {
                    process.stop()?;
                    self.queue.execute(Command::Nack {
                        lease,
                        delay_ms: self.settings.retry_delay_ms,
                        reason: error.to_string(),
                    })?;
                    return Err(error);
                }
            }
            // Renew before waiting too: startup time must not consume the next heartbeat interval.
            if let Err(error) = self.queue.execute(Command::Renew {
                lease: lease.clone(),
                lease_ms: self.settings.claim.lease_ms,
            }) {
                process.stop()?;
                return Err(error);
            }
            self.wait
                .wait_ms((self.settings.claim.lease_ms as u64 / 3).clamp(1, 1000));
        }
    }
    fn idle(&mut self) {
        self.wait.wait_ms(self.settings.idle_ms);
    }
    fn stopped(&self) -> bool {
        self.wait.stopped()
    }
}
