use crate::{io, platform};
use anyq_queue_api::Result;
use anyq_worker_execution::Wait;
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};
pub struct SystemWait {
    cancelled: Arc<AtomicBool>,
    _interrupt: platform::Interrupt,
}
impl SystemWait {
    pub fn new() -> Result<Self> {
        let cancelled = Arc::new(AtomicBool::new(false));
        let interrupt = platform::Interrupt::install(cancelled.clone()).map_err(io)?;
        Ok(Self {
            cancelled,
            _interrupt: interrupt,
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
