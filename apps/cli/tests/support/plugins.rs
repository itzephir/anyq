use crate::support::queue::{Queue, wait_for_file};
use serde_json::Value;
use std::{
    fs,
    path::Path,
    thread,
    time::{Duration, Instant},
};

fn enable(queue: &Queue, name: &str, mode: &str, log: &Path) {
    queue.call(&[
        "plugin",
        "enable",
        name,
        env!("CARGO_BIN_EXE_anyq-plugin-fixture"),
        mode,
        log.to_str().unwrap(),
    ]);
    wait(|| {
        status(queue, name)["running"] == true
            || status(queue, name)["failures"].as_u64().unwrap() > 0
    });
}

fn status(queue: &Queue, name: &str) -> Value {
    serde_json::from_str(&queue.call(&["plugin", "status", name])).unwrap()
}

fn wait(mut condition: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(15);
    while !condition() {
        assert!(Instant::now() < deadline, "plugin condition timed out");
        thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn explicitly_enabled_consumers_start_at_future_watermark_and_retry_stable_event_ids_independently()
{
    let queue = Queue::new();
    queue.topic("cpu");
    queue.task("old", &["cpu"]);
    let good = queue.log("good");
    let retry = queue.log("retry");
    enable(&queue, "good", "ack", &good);
    enable(&queue, "retry", "crash-once", &retry);
    queue.call(&["task", "complete", "old"]);
    queue.task("new", &["cpu"]);

    wait(|| fs::read_to_string(&retry).is_ok_and(|text| text.lines().count() >= 2));
    wait(|| status(&queue, "retry")["lag"] == 0 && status(&queue, "good")["lag"] == 0);
    let messages = fs::read_to_string(&retry)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .collect::<Vec<_>>();

    assert_eq!(messages[0]["id"], messages[1]["id"]);
    assert_eq!(messages[0]["task"], "new");
    assert_eq!(fs::read_to_string(good).unwrap().lines().count(), 1);
    assert_eq!(queue.state("new"), "running");
    assert_eq!(queue.usage("cpu"), 1);
}

#[test]
fn lost_completion_response_replays_durable_result_without_duplicate_events() {
    let queue = Queue::new();
    queue.topic("cpu");
    let log = queue.log("complete");
    enable(&queue, "complete", "complete-lost", &log);
    queue.task("job", &["cpu"]);

    wait(|| status(&queue, "complete")["lag"] == 0 && queue.state("job") == "completed");
    let connection = rusqlite::Connection::open(&queue.db).unwrap();
    let count:i64=connection.query_row("SELECT count(*) FROM queue_events WHERE json_extract(payload,'$.kind')='task.completed' AND json_extract(payload,'$.task.name')='job'",[],|row|row.get(0)).unwrap();

    assert_eq!(count, 1);
    assert_eq!(queue.usage("cpu"), 0);
    assert!(fs::read_to_string(log).unwrap().lines().count() >= 2);
}

#[test]
fn manual_completion_cancels_managed_worker_before_admitting_successor() {
    let queue = Queue::new();
    queue.topic("cpu");
    let log = queue.log("managed");
    enable(&queue, "managed", "managed", &log);
    queue.task("owner", &["cpu"]);
    wait_for_file(&log.with_extension("marker"), Some("started"));
    queue.task("next", &["cpu"]);

    queue.call(&["task", "complete", "owner"]);

    assert_eq!(
        fs::read_to_string(log.with_extension("stopped")).unwrap(),
        "owner"
    );
    assert_eq!(queue.state("next"), "running");
    assert_eq!(queue.usage("cpu"), 1);
}

#[test]
fn uncertain_worker_retains_slots_and_requires_explicit_confirmed_cleanup() {
    let queue = Queue::new();
    queue.topic("cpu");
    let log = queue.log("uncertain");
    enable(&queue, "uncertain", "uncertain", &log);
    queue.task("owner", &["cpu"]);
    wait_for_file(&log.with_extension("marker"), Some("started"));
    queue.task("next", &["cpu"]);

    assert!(!queue.raw(&["task", "complete", "owner"]).status.success());
    assert_eq!(queue.state("owner"), "running");
    assert_eq!(queue.state("next"), "queued");
    let work: Vec<Value> =
        serde_json::from_str(&queue.call(&["plugin", "inspect", "uncertain"])).unwrap();
    assert_eq!(work[0]["state"], "uncertain");
    let id = work[0]["action_id"].as_str().unwrap();
    assert!(
        !queue
            .raw(&["plugin", "resolve", "uncertain", id])
            .status
            .success()
    );
    queue.call(&["plugin", "disable", "uncertain"]);
    queue.call(&["plugin", "resolve", "uncertain", id, "--confirmed-cleanup"]);
    queue.call(&["task", "complete", "owner"]);

    assert_eq!(queue.state("next"), "running");
}

#[test]
fn timer_ack_state_and_original_deadline_survive_disabled_daemon_restart() {
    let queue = Queue::new();
    let clock = queue.dir.path().join("clock.txt");
    fs::write(&clock, "0").unwrap();
    start_with_clock(&queue, &clock);
    queue.topic("cpu");
    let log = queue.log("timer");
    enable(&queue, "timer", "timer", &log);
    queue.task("owner", &["cpu"]);
    wait(|| status(&queue, "timer")["active_timers"] == 1);
    let connection = rusqlite::Connection::open(&queue.db).unwrap();
    let deadline: i64 = connection
        .query_row(
            "SELECT deadline FROM plugin_timers WHERE plugin='timer'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    queue.call(&["plugin", "disable", "timer"]);
    queue.call(&["daemon", "stop"]);
    let next_clock = clock.with_extension("next");
    fs::write(&next_clock, (deadline + 1).to_string()).unwrap();
    fs::rename(next_clock, &clock).unwrap();
    start_with_clock(&queue, &clock);
    assert_eq!(status(&queue, "timer")["enabled"], false);
    assert_eq!(status(&queue, "timer")["active_timers"], 1);

    enable(&queue, "timer", "timer", &log);
    wait(|| status(&queue, "timer")["active_timers"] == 0);
    let timer: Value = fs::read_to_string(&log)
        .unwrap()
        .lines()
        .find_map(|line| {
            let value: Value = serde_json::from_str(line).unwrap();
            value.get("timer").is_some().then_some(value)
        })
        .unwrap();

    assert_eq!(timer["deadline"], deadline);
    assert_eq!(queue.state("owner"), "running");
}

#[test]
fn incompatible_hanging_and_flooding_plugins_are_visible_without_blocking_queue_clients() {
    let queue = Queue::new();
    queue.topic("cpu");
    for mode in ["incompatible", "no-handshake", "flood", "bad-state"] {
        enable(&queue, mode, mode, &queue.log(mode));
    }
    let started = Instant::now();
    queue.task("owner", &["cpu"]);
    assert!(started.elapsed() < Duration::from_secs(3));
    for mode in ["incompatible", "no-handshake", "flood", "bad-state"] {
        wait(|| status(&queue, mode)["last_failure"].is_string());
        let failure = status(&queue, mode)["last_failure"]
            .as_str()
            .unwrap()
            .to_string();
        assert!(failure.len() < 128);
        assert!(!failure.contains("SECRET"));
        assert!(!failure.contains("sensitive"));
        queue.call(&["plugin", "disable", mode]);
    }

    assert_eq!(queue.state("owner"), "running");
    assert_eq!(queue.usage("cpu"), 1);
}

#[test]
fn failed_disable_cancellation_remains_inspectable_and_preserves_all_slots() {
    let queue = Queue::new();
    queue.topic("cpu");
    let log = queue.log("disable-uncertain");
    enable(&queue, "worker", "uncertain", &log);
    queue.task("owner", &["cpu"]);
    wait_for_file(&log.with_extension("marker"), Some("started"));
    queue.task("next", &["cpu"]);

    queue.call(&["plugin", "disable", "worker"]);
    wait(|| !status(&queue, "worker")["running"].as_bool().unwrap());

    let status = status(&queue, "worker");
    assert_eq!(status["unresolved_actions"], 1);
    assert!(status["last_failure"].is_string());
    assert_eq!(queue.state("owner"), "running");
    assert_eq!(queue.state("next"), "queued");
    assert_eq!(queue.usage("cpu"), 1);
}

#[test]
fn daemon_stop_waits_for_managed_drain_before_an_immediate_restart() {
    let queue = Queue::new();
    queue.topic("cpu");
    let log = queue.log("stop-and-restart");
    enable(&queue, "worker", "uncertain", &log);
    queue.task("owner", &["cpu"]);
    wait_for_file(&log.with_extension("marker"), Some("started"));
    queue.task("next", &["cpu"]);

    queue.call(&["daemon", "stop"]);
    assert_eq!(queue.call(&["task", "owner", "meta"]), "{}");

    assert_eq!(queue.state("owner"), "running");
    assert_eq!(queue.state("next"), "queued");
    assert_eq!(queue.usage("cpu"), 1);
    queue.call(&["plugin", "disable", "worker"]);
}

fn start_with_clock(queue: &Queue, clock: &Path) {
    use std::process::Stdio;

    let output = crate::support::process::TestProcess::spawn(
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

#[test]
fn slow_managed_cancellation_is_not_flooded_or_killed_before_confirmed_stop() {
    let queue = Queue::new();
    queue.topic("cpu");
    let log = queue.log("slow-worker");
    enable(&queue, "slow", "managed-slow", &log);
    queue.task("owner", &["cpu"]);
    wait_for_file(&log.with_extension("marker"), Some("started"));
    queue.task("next", &["cpu"]);

    queue.call(&["task", "complete", "owner"]);

    assert_eq!(
        fs::read_to_string(log.with_extension("stopped")).unwrap(),
        "owner"
    );
    assert_eq!(status(&queue, "slow")["failures"], 0);
    assert_eq!(queue.state("next"), "running");
}
