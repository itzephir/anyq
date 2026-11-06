use super::trace::Trace;
use crate::Process;
use anyq_queue_api::Result;
use std::collections::VecDeque;
pub struct TestProcess {
    pub trace: Trace,
    pub polls: VecDeque<Result<Option<i32>>>,
}
impl Process for TestProcess {
    fn poll(&mut self) -> Result<Option<i32>> {
        self.polls.pop_front().expect("unexpected poll")
    }
    fn stop(&mut self) -> Result<()> {
        self.trace.stopped.set(true);
        Ok(())
    }
}
