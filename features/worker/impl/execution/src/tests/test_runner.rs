use super::{test_process::TestProcess, trace::Trace};
use crate::{Process, Runner};
use anyq_queue_api::{Assignment, Result};
use std::collections::VecDeque;
pub struct TestRunner {
    pub trace: Trace,
    pub polls: VecDeque<Result<Option<i32>>>,
}
impl Runner for TestRunner {
    fn start(&mut self, _: &Assignment) -> Result<Box<dyn Process>> {
        Ok(Box::new(TestProcess {
            trace: self.trace.clone(),
            polls: std::mem::take(&mut self.polls),
        }))
    }
}
