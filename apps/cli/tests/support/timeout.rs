use crate::support::{
    process::TestProcess,
    queue::{Queue, wait_for_file},
};
use anyq_queue_api::{Metadata, MetadataValue};
use serde_json::Value;
use std::{
    fs,
    path::Path,
    process::Stdio,
    thread,
    time::{Duration, Instant},
};

fn wait(mut predicate: impl FnMut() -> bool) {
    let end = Instant::now() + Duration::from_secs(15);
    while !predicate() {
        assert!(Instant::now() < end, "timeout plugin condition not reached");
        thread::sleep(Duration::from_millis(20));
    }
}

fn status(queue: &Queue) -> Value {
    serde_json::from_str(&queue.call(&["plugin", "status", "timeout"])).unwrap()
}

fn state(queue: &Queue) -> Metadata {
    let dto: anyq_queue_wire::MetadataDto =
        serde_json::from_value(status(queue)["state"].clone()).unwrap();
    dto.try_into().unwrap()
}

fn clock(queue: &Queue, now: i64) -> std::path::PathBuf {
    let path = queue.dir.path().join("clock");
    let next = path.with_extension("next");
    fs::write(&next, now.to_string()).unwrap();
    fs::rename(next, &path).unwrap();
    path
}

fn start(queue: &Queue, clock: &Path) {
    let output = TestProcess::spawn(
        queue
            .cmd(&["daemon", "start"])
            .env("ANYQ_TEST_CLOCK_FILE", clock)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped()),
    )
    .output();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn enable(queue: &Queue) {
    queue.call(&[
        "plugin",
        "enable",
        "timeout",
        env!("CARGO_BIN_EXE_anyq"),
        "plugin-runtime",
        "timeout",
    ]);
    wait(|| status(queue)["running"] == true);
}

#[test]
fn minimum_snapshot_deadline_survives_disable_edits_and_overdue_restart() {
    let queue = Queue::new();
    let path = clock(&queue, 1000);
    start(&queue, &path);
    queue.topic("cpu");
    queue.call(&["topic", "cpu", "meta", "timeout", "30m"]);
    enable(&queue);
    queue.call(&[
        "task", "create", "owner", "-t", "cpu", "--meta", "timeout", "2h",
    ]);
    queue.task("next", &["cpu"]);
    wait(|| status(&queue)["active_timers"] == 1);
    let saved = state(&queue);
    let MetadataValue::Object(active) = saved.get(Some("active")).unwrap() else {
        panic!()
    };
    let plan = anyq_timeout_api::Deadline::restore(
        Metadata::new(active.values().next().unwrap().clone()).unwrap(),
    )
    .unwrap();
    assert_eq!(plan.deadline, 1_801_000);
    assert_eq!(plan.source, "topic:cpu");
    queue.call(&["topic", "cpu", "meta", "timeout", "1d"]);
    queue.call(&["plugin", "disable", "timeout"]);
    queue.call(&["daemon", "stop"]);
    clock(&queue, plan.deadline + 1);
    start(&queue, &path);
    assert_eq!(queue.state("owner"), "running");
    assert_eq!(status(&queue)["active_timers"], 1);

    enable(&queue);
    wait(|| queue.state("owner") == "completed");

    assert_eq!(queue.state("next"), "running");
    assert_eq!(queue.usage("cpu"), 1);
}

#[test]
fn invalid_timeout_is_inspectable_and_does_not_block_other_tasks_or_force_queued_work() {
    let queue = Queue::new();
    let path = clock(&queue, 0);
    start(&queue, &path);
    queue.topic("cpu");
    enable(&queue);
    queue.call(&[
        "task", "create", "bad", "-t", "cpu", "--meta", "timeout", "0h",
    ]);
    queue.call(&[
        "task", "create", "queued", "-t", "cpu", "--meta", "timeout", "1ms",
    ]);
    queue.call(&["task", "create", "registered", "--meta", "timeout", "1ms"]);
    wait(|| state(&queue).get(Some("last_error")).is_ok());
    let error = format!("{:?}", state(&queue).get(Some("last_error")).unwrap());

    clock(&queue, 100_000);
    queue.topic("other");
    queue.task("ordinary", &["other"]);
    wait(|| status(&queue)["lag"] == 0);

    assert!(error.contains("task:bad"));
    assert!(error.contains("0h"));
    assert_eq!(queue.state("bad"), "running");
    assert_eq!(queue.state("queued"), "queued");
    assert_eq!(queue.state("registered"), "registered");
    assert_eq!(queue.state("ordinary"), "running");
    assert_eq!(status(&queue)["active_timers"], 0);
}

#[test]
fn manual_completion_and_expiry_have_one_completed_event_and_retire_timer() {
    let queue = Queue::new();
    let path = clock(&queue, 0);
    start(&queue, &path);
    queue.topic("cpu");
    enable(&queue);
    queue.call(&[
        "task", "create", "owner", "-t", "cpu", "--meta", "timeout", "10ms",
    ]);
    queue.task("next", &["cpu"]);
    wait(|| status(&queue)["active_timers"] == 1);

    clock(&queue, 11);
    let _ = queue.raw(&["task", "complete", "owner"]);
    wait(|| queue.state("owner") == "completed" && status(&queue)["active_timers"] == 0);
    let count: i64 = rusqlite::Connection::open(&queue.db).unwrap().query_row("SELECT count(*) FROM queue_events WHERE json_extract(payload,'$.kind')='task.completed' AND json_extract(payload,'$.task.name')='owner'", [], |row| row.get(0)).unwrap();

    assert_eq!(count, 1);
    assert_eq!(queue.state("next"), "running");
}

#[test]
fn confirmed_worker_cancellation_allows_expiry_but_uncertain_worker_keeps_every_slot() {
    for mode in ["managed", "uncertain"] {
        let queue = Queue::new();
        let path = clock(&queue, 0);
        start(&queue, &path);
        queue.topic("cpu");
        let log = queue.log("worker");
        queue.call(&[
            "plugin",
            "enable",
            "worker",
            env!("CARGO_BIN_EXE_anyq-plugin-fixture"),
            mode,
            log.to_str().unwrap(),
        ]);
        wait(|| {
            serde_json::from_str::<Value>(&queue.call(&["plugin", "status", "worker"])).unwrap()["running"]
                == true
        });
        enable(&queue);
        queue.call(&[
            "task", "create", "owner", "-t", "cpu", "--meta", "timeout", "10ms",
        ]);
        wait_for_file(&log.with_extension("marker"), Some("started"));
        queue.task("next", &["cpu"]);
        wait(|| status(&queue)["active_timers"] == 1);

        clock(&queue, 11);
        wait_for_file(
            &log.with_extension("marker"),
            Some(if mode == "managed" {
                "stopped"
            } else {
                "uncertain"
            }),
        );

        if mode == "managed" {
            wait(|| queue.state("owner") == "completed");
            assert_eq!(queue.state("next"), "running");
        } else {
            wait(|| state(&queue).get(Some("last_error")).is_ok());
            assert_eq!(queue.state("owner"), "running");
            assert_eq!(queue.state("next"), "queued");
            assert_eq!(queue.usage("cpu"), 1);
            queue.call(&["plugin", "disable", "timeout"]);
            queue.call(&["plugin", "disable", "worker"]);
        }
    }
}

#[test]
fn disabled_plugin_does_not_apply_to_preexisting_running_events() {
    let queue = Queue::new();
    let path = clock(&queue, 0);
    start(&queue, &path);
    queue.topic("cpu");
    queue.call(&[
        "task", "create", "owner", "-t", "cpu", "--meta", "timeout", "1ms",
    ]);
    clock(&queue, 10);

    enable(&queue);
    queue.task("next", &["cpu"]);
    wait(|| status(&queue)["lag"] == 0);

    assert_eq!(queue.state("owner"), "running");
    assert_eq!(status(&queue)["active_timers"], 0);
}
