mod claim_stub;
mod completion_spy;
mod harness;
mod rejection_spy;
mod renew_stub;
mod test_process;
mod test_runner;
mod test_wait;
mod topic_stub;
mod trace;
use anyq_queue_api::*;
use anyq_worker_api::{Run, Settings, Worker};
use harness::Harness;
use std::collections::VecDeque;
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
    let mut h = Harness::new(
        VecDeque::from([Ok(None), Ok(None), Ok(Some(0))]),
        false,
        None,
        None,
    );
    assert_eq!(h.worker.run_one().unwrap(), Run::Succeeded(1));
    assert_eq!(h.trace.renewals.get(), 2);
    assert_eq!(h.trace.ticks.get(), 2);
    assert!(!h.trace.stopped.get());
    assert_eq!(h.trace.completions.borrow().len(), 1);
    assert!(h.trace.rejections.borrow().is_empty());
}
#[test]
fn lease_loss_stops_process_and_never_acknowledges() {
    let mut h = Harness::new(VecDeque::from([Ok(None)]), true, None, None);
    assert_eq!(h.worker.run_one(), Err(Error::LeaseLost));
    assert!(h.trace.stopped.get());
    assert!(h.trace.completions.borrow().is_empty());
    assert!(h.trace.rejections.borrow().is_empty());
}
#[test]
fn nonzero_exit_returns_delivery_without_publishing_followup() {
    let next = Publish {
        topic: "emulator".into(),
        payload: "{}".into(),
        key: None,
        max_attempts: 3,
    };
    let mut h = Harness::new(VecDeque::from([Ok(Some(17))]), false, None, Some(next));
    assert_eq!(h.worker.run_one().unwrap(), Run::Failed(1));
    assert!(h.trace.completions.borrow().is_empty());
    assert!(h.trace.rejections.borrow()[0].reason.contains("17"));
}
#[test]
fn cancellation_stops_command_before_releasing_delivery() {
    let mut h = Harness::new(VecDeque::from([Ok(None)]), false, Some(1), None);
    assert_eq!(h.worker.run_one().unwrap(), Run::Stopped);
    assert!(h.trace.stopped.get());
    assert_eq!(h.trace.rejections.borrow().len(), 1);
    assert!(h.trace.completions.borrow().is_empty());
}
