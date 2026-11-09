use crate::support::{large_output::seed_tasks, process::TestProcess, queue::Queue};
use std::process::Stdio;

#[test]
fn large_stdout_is_complete_when_consumed_and_does_not_change_existing_ownership() {
    let queue = Queue::new();
    let mut expected = seed_tasks(&queue);
    queue.topic("cpu");
    queue.task("owner", &["cpu"]);
    queue.call(&["task", "wait", "owner"]);
    expected.push_str("- owner\n");

    let output = TestProcess::spawn(
        queue
            .cmd(&["task", "list"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped()),
    )
    .output();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, expected.as_bytes());
    assert_eq!(queue.state("owner"), "running");
    assert_eq!(queue.usage("cpu"), 1);
}

#[test]
fn closed_stdout_pipe_returns_an_error_without_releasing_existing_ownership() {
    let queue = Queue::new();
    seed_tasks(&queue);
    queue.topic("cpu");
    queue.task("owner", &["cpu"]);
    queue.call(&["task", "wait", "owner"]);
    let mut child = TestProcess::spawn(
        queue
            .cmd(&["task", "list"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped()),
    );

    drop(child.child.as_mut().unwrap().stdout.take());
    let output = child.output();

    assert_eq!(output.status.code(), Some(2));
    assert!(!output.stderr.is_empty());
    assert_eq!(queue.state("owner"), "running");
    assert_eq!(queue.usage("cpu"), 1);

    queue.call(&["task", "owner", "complete"]);

    assert_eq!(queue.state("owner"), "completed");
    assert_eq!(queue.usage("cpu"), 0);
}
