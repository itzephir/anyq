use super::trace::Trace;
use crate::Wait;

pub struct TestWait {
    pub trace: Trace,
    pub cancel_after: Option<u32>,
}

impl Wait for TestWait {
    fn wait_ms(&mut self, _: u64) {
        self.trace.ticks.set(self.trace.ticks.get() + 1);
    }

    fn stopped(&self) -> bool {
        self.cancel_after
            .is_some_and(|n| self.trace.ticks.get() >= n)
    }
}
