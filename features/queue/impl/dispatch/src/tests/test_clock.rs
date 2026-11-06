use crate::ports::Clock;
use std::{cell::Cell, rc::Rc};

#[derive(Clone, Default)]
pub struct TestClock(pub Rc<Cell<i64>>);

impl Clock for TestClock {
    fn now_ms(&self) -> i64 {
        self.0.get()
    }
}
