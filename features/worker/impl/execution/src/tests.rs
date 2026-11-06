use super::*;
use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    rc::Rc,
};
struct TestQueue {
    calls: Rc<RefCell<Vec<Command>>>,
    outputs: VecDeque<Result<Output>>,
}
impl Queue for TestQueue {
    fn execute(&mut self, command: Command) -> Result<Output> {
        self.calls.borrow_mut().push(command);
        self.outputs.pop_front().expect("unexpected queue call")
    }
}
struct TestRunner {
    stopped: Rc<Cell<bool>>,
    polls: VecDeque<Result<Option<i32>>>,
}
struct TestProcess {
    stopped: Rc<Cell<bool>>,
    polls: VecDeque<Result<Option<i32>>>,
}
impl Process for TestProcess {
    fn poll(&mut self) -> Result<Option<i32>> {
        self.polls.pop_front().expect("unexpected process poll")
    }
    fn stop(&mut self) -> Result<()> {
        self.stopped.set(true);
        Ok(())
    }
}
impl Runner for TestRunner {
    fn start(&mut self, _: &Assignment) -> Result<Box<dyn Process>> {
        Ok(Box::new(TestProcess {
            stopped: self.stopped.clone(),
            polls: std::mem::take(&mut self.polls),
        }))
    }
}
struct TestWait {
    ticks: Rc<Cell<u32>>,
    cancel_after: Option<u32>,
}
impl Wait for TestWait {
    fn wait_ms(&mut self, _: u64) {
        self.ticks.set(self.ticks.get() + 1);
    }
    fn stopped(&self) -> bool {
        self.cancel_after.is_some_and(|n| self.ticks.get() >= n)
    }
}
fn assignment() -> Assignment {
    let message = Message {
        id: 1,
        topic: "builds".into(),
        payload: "{}".into(),
        key: None,
        max_attempts: 3,
        created_at: 0,
    };
    let mut delivery = Delivery::pending(&message, "builders".into());
    delivery
        .claim("worker".into(), "token".into(), 0, 1000)
        .unwrap();
    Assignment { message, delivery }
}
fn settings() -> Settings {
    Settings {
        claim: Claim {
            topic: "builds".into(),
            group: "builders".into(),
            consumer: "worker".into(),
            lease_ms: 1000,
        },
        retry_delay_ms: 10,
        idle_ms: 10,
        next: None,
    }
}
#[test]
fn long_command_renews_until_success_then_acks() {
    let a = assignment();
    let calls = Rc::new(RefCell::new(Vec::new()));
    let stopped = Rc::new(Cell::new(false));
    let ticks = Rc::new(Cell::new(0));
    let queue = TestQueue {
        calls: calls.clone(),
        outputs: VecDeque::from([
            Ok(Output::Claimed(Some(a.clone()))),
            Ok(Output::Delivery(a.delivery.clone())),
            Ok(Output::Delivery(a.delivery.clone())),
            Ok(Output::Acked {
                delivery: a.delivery,
                next: None,
            }),
        ]),
    };
    let runner = TestRunner {
        stopped: stopped.clone(),
        polls: VecDeque::from([Ok(None), Ok(None), Ok(Some(0))]),
    };
    let mut w = Execution::new(
        Box::new(queue),
        runner,
        TestWait {
            ticks: ticks.clone(),
            cancel_after: None,
        },
        settings(),
    )
    .unwrap();
    assert_eq!(w.run_one().unwrap(), Run::Succeeded(1));
    assert_eq!(ticks.get(), 2);
    assert!(!stopped.get());
    assert!(matches!(
        calls.borrow().as_slice(),
        [
            Command::Claim(_),
            Command::Renew { .. },
            Command::Renew { .. },
            Command::Ack { .. }
        ]
    ));
}
#[test]
fn lease_loss_stops_process_and_never_acknowledges() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let stopped = Rc::new(Cell::new(false));
    let queue = TestQueue {
        calls: calls.clone(),
        outputs: VecDeque::from([
            Ok(Output::Claimed(Some(assignment()))),
            Err(Error::LeaseLost),
        ]),
    };
    let runner = TestRunner {
        stopped: stopped.clone(),
        polls: VecDeque::from([Ok(None)]),
    };
    let mut w = Execution::new(
        Box::new(queue),
        runner,
        TestWait {
            ticks: Rc::new(Cell::new(0)),
            cancel_after: None,
        },
        settings(),
    )
    .unwrap();
    assert_eq!(w.run_one(), Err(Error::LeaseLost));
    assert!(stopped.get());
    assert_eq!(calls.borrow().len(), 2);
}
#[test]
fn nonzero_exit_returns_delivery_without_publishing_followup() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let a = assignment();
    let queue = TestQueue {
        calls: calls.clone(),
        outputs: VecDeque::from([
            Ok(Output::Groups(vec![])),
            Ok(Output::Claimed(Some(a.clone()))),
            Ok(Output::Delivery(a.delivery)),
        ]),
    };
    let runner = TestRunner {
        stopped: Rc::new(Cell::new(false)),
        polls: VecDeque::from([Ok(Some(17))]),
    };
    let mut config = settings();
    config.next = Some(Publish {
        topic: "emulator".into(),
        payload: "{}".into(),
        key: None,
        max_attempts: 3,
    });
    let mut w = Execution::new(
        Box::new(queue),
        runner,
        TestWait {
            ticks: Rc::new(Cell::new(0)),
            cancel_after: None,
        },
        config,
    )
    .unwrap();
    assert_eq!(w.run_one().unwrap(), Run::Failed(1));
    assert!(
        matches!(calls.borrow().last(),Some(Command::Nack { reason,.. }) if reason.contains("17"))
    );
}
#[test]
fn cancellation_stops_command_before_releasing_delivery() {
    let a = assignment();
    let calls = Rc::new(RefCell::new(Vec::new()));
    let stopped = Rc::new(Cell::new(false));
    let queue = TestQueue {
        calls: calls.clone(),
        outputs: VecDeque::from([
            Ok(Output::Claimed(Some(a.clone()))),
            Ok(Output::Delivery(a.delivery.clone())),
            Ok(Output::Delivery(a.delivery)),
        ]),
    };
    let runner = TestRunner {
        stopped: stopped.clone(),
        polls: VecDeque::from([Ok(None)]),
    };
    let mut w = Execution::new(
        Box::new(queue),
        runner,
        TestWait {
            ticks: Rc::new(Cell::new(0)),
            cancel_after: Some(1),
        },
        settings(),
    )
    .unwrap();
    assert_eq!(w.run_one().unwrap(), Run::Stopped);
    assert!(stopped.get());
    assert!(matches!(calls.borrow().last(), Some(Command::Nack { .. })));
}
