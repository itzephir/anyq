use crate::{ExecutionPorts, Runner, Wait};
use anyq_queue_api::{CompleteDeliveryRequest, RejectDeliveryRequest, Result};
use anyq_worker_api::{Run, Settings, Worker};
pub struct Execution<R, W> {
    ports: ExecutionPorts,
    runner: R,
    wait: W,
    settings: Settings,
}
impl<R, W> Execution<R, W> {
    pub fn new(ports: ExecutionPorts, runner: R, wait: W, settings: Settings) -> Result<Self> {
        settings.validate()?;
        Ok(Self {
            ports,
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
            self.ports.topics.get(&next.topic)?;
        }
        let assignment = self.ports.claim.claim(self.settings.claim.clone())?;
        let Some(assignment) = assignment else {
            return Ok(Run::Idle);
        };
        let lease = assignment.lease();
        let mut process = match self.runner.start(&assignment) {
            Ok(p) => p,
            Err(error) => {
                self.ports.reject.reject(RejectDeliveryRequest {
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
                self.ports.reject.reject(RejectDeliveryRequest {
                    lease,
                    delay_ms: self.settings.retry_delay_ms,
                    reason: "worker interrupted".into(),
                })?;
                return Ok(Run::Stopped);
            }
            match process.poll() {
                Ok(Some(code)) => {
                    if code == 0 {
                        self.ports.complete.complete(CompleteDeliveryRequest {
                            lease,
                            result: Some("{\"exit_code\":0}".into()),
                            next: self.settings.next.clone(),
                        })?;
                        return Ok(Run::Succeeded(assignment.message.id));
                    }
                    self.ports.reject.reject(RejectDeliveryRequest {
                        lease,
                        delay_ms: self.settings.retry_delay_ms,
                        reason: format!("command exited with code {code}"),
                    })?;
                    return Ok(Run::Failed(assignment.message.id));
                }
                Ok(None) => {}
                Err(error) => {
                    process.stop()?;
                    self.ports.reject.reject(RejectDeliveryRequest {
                        lease,
                        delay_ms: self.settings.retry_delay_ms,
                        reason: error.to_string(),
                    })?;
                    return Err(error);
                }
            }
            // Renew before waiting too: startup time must not consume the next heartbeat interval.
            if let Err(error) = self
                .ports
                .renew
                .renew(lease.clone(), self.settings.claim.lease_ms)
            {
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
