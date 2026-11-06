use super::{
    assignment, claim_stub::ClaimStub, completion_spy::CompletionSpy, rejection_spy::RejectionSpy,
    renew_stub::RenewStub, settings, test_runner::TestRunner, test_wait::TestWait,
    topic_stub::TopicStub, trace::Trace,
};

use crate::{Execution, ExecutionPorts};
use anyq_queue_api::{Publish, Result};
use std::collections::VecDeque;

pub struct Harness {
    pub worker: Execution<TestRunner, TestWait>,
    pub trace: Trace,
}

impl Harness {
    pub fn new(
        polls: VecDeque<Result<Option<i32>>>,
        lose_lease: bool,
        cancel_after: Option<u32>,
        next: Option<Publish>,
    ) -> Self {
        let trace = Trace::default();
        let ports = ExecutionPorts {
            claim: Box::new(ClaimStub(Some(assignment()))),
            renew: Box::new(RenewStub {
                trace: trace.clone(),
                fail: lose_lease,
            }),
            complete: Box::new(CompletionSpy(trace.clone())),
            reject: Box::new(RejectionSpy(trace.clone())),
            topics: Box::new(TopicStub),
        };

        let mut config = settings();
        config.next = next;
        let worker = Execution::new(
            ports,
            TestRunner {
                trace: trace.clone(),
                polls,
            },
            TestWait {
                trace: trace.clone(),
                cancel_after,
            },
            config,
        )
        .unwrap();
        Self { worker, trace }
    }
}
