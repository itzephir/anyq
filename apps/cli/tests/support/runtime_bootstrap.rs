use crate::support::{
    process::TestProcess,
    queue::{Queue, wait_for_file},
};
use rusqlite::Connection;
use serde_json::Value;
use std::{
    fs,
    path::Path,
    process::Stdio,
    thread,
    time::{Duration, Instant},
};

fn wait(mut condition: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(15);
    while !condition() {
        assert!(
            Instant::now() < deadline,
            "runtime bootstrap condition timed out"
        );
        thread::sleep(Duration::from_millis(20));
    }
}

fn clock(queue: &Queue, now: i64) -> std::path::PathBuf {
    let path = queue.dir.path().join("clock");
    let next = path.with_extension("next");
    fs::write(&next, now.to_string()).unwrap();
    fs::rename(next, &path).unwrap();
    path
}

// Only lifecycle Ping requests are allowed here: opening a queue session would
// initialize storage and hide a runtime that still waits for the first RPC.
fn owner(queue: &Queue, clock: &Path) -> TestProcess {
    let mut command = queue.cmd(&["daemon", "run"]);
    command
        .env("ANYQ_TEST_CLOCK_FILE", clock)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x00000010);
    }
    let mut process = TestProcess::spawn(&mut command);
    wait(|| {
        process.assert_running();
        queue.call(&["daemon", "status"]) != "daemon is stopped"
    });
    process
}

fn stop(queue: &Queue, process: TestProcess) {
    queue.call(&["daemon", "stop"]);
    assert!(process.output().status.success());
}

fn count(queue: &Queue, sql: &str) -> i64 {
    let connection = Connection::open(&queue.db).unwrap();
    connection.busy_timeout(Duration::from_secs(5)).unwrap();
    connection.query_row(sql, [], |row| row.get(0)).unwrap()
}

fn timeout(queue: &Queue) {
    queue.topic("cpu");
    queue.call(&[
        "plugin",
        "enable",
        "timeout",
        env!("CARGO_BIN_EXE_anyq"),
        "plugin-runtime",
        "timeout",
    ]);
    wait(|| count(queue, "SELECT running FROM plugins WHERE name='timeout'") == 1);
    queue.call(&[
        "task", "create", "owner", "-t", "cpu", "--meta", "timeout", "10ms",
    ]);
    queue.task("next", &["cpu"]);
    wait(|| {
        count(
            queue,
            "SELECT count(*) FROM plugin_timers WHERE plugin='timeout'",
        ) == 1
    });
}

#[test]
fn runtime_bootstrap_recovers_overdue_timeout_without_any_queue_rpc() {
    let queue = Queue::new();
    let path = clock(&queue, 0);
    let first = owner(&queue, &path);
    timeout(&queue);
    let deadline = count(
        &queue,
        "SELECT deadline FROM plugin_timers WHERE plugin='timeout'",
    );
    assert_eq!(deadline, 10);
    stop(&queue, first);
    clock(&queue, deadline + 1);

    let second = owner(&queue, &path);
    wait(|| queue.state("owner") == "completed" && queue.state("next") == "running");
    wait(|| {
        count(
            &queue,
            "SELECT count(*) FROM plugin_timers WHERE plugin='timeout'",
        ) == 0
    });

    assert_eq!(
        count(
            &queue,
            "SELECT count(*) FROM completion_receipts WHERE owner='timeout' AND task_name='owner'"
        ),
        1
    );
    assert_eq!(
        count(
            &queue,
            "SELECT count(*) FROM queue_events WHERE json_extract(payload,'$.kind')='task.completed' AND json_extract(payload,'$.task.name')='owner'"
        ),
        1
    );
    assert_eq!(
        count(
            &queue,
            "SELECT count(*) FROM queue_events WHERE json_extract(payload,'$.kind')='task.running'"
        ),
        2
    );
    stop(&queue, second);

    let third = owner(&queue, &path);
    wait(|| count(&queue, "SELECT running FROM plugins WHERE name='timeout'") == 1);
    assert_eq!(
        count(
            &queue,
            "SELECT count(*) FROM completion_receipts WHERE owner='timeout' AND task_name='owner'"
        ),
        1
    );
    assert_eq!(
        count(
            &queue,
            "SELECT count(*) FROM queue_events WHERE json_extract(payload,'$.kind')='task.completed'"
        ),
        1
    );
    stop(&queue, third);
}

#[test]
fn runtime_bootstrap_preserves_disabled_deadlines_and_does_not_launch_plugin() {
    let queue = Queue::new();
    let path = clock(&queue, 0);
    let first = owner(&queue, &path);
    timeout(&queue);
    queue.call(&["plugin", "disable", "timeout"]);
    stop(&queue, first);
    clock(&queue, 100);
    Connection::open(&queue.db)
        .unwrap()
        .execute("UPDATE plugins SET running=1 WHERE name='timeout'", [])
        .unwrap();

    let second = owner(&queue, &path);
    // Bootstrap resets stale lifecycle state even for disabled registrations.
    // A sentinel makes this a positive startup assertion, not a timing guess.
    wait(|| count(&queue, "SELECT running FROM plugins WHERE name='timeout'") == 0);
    assert_eq!(queue.state("owner"), "running");
    assert_eq!(queue.state("next"), "queued");
    assert_eq!(
        count(&queue, "SELECT enabled FROM plugins WHERE name='timeout'"),
        0
    );
    assert_eq!(
        count(
            &queue,
            "SELECT deadline FROM plugin_timers WHERE plugin='timeout'"
        ),
        10
    );
    assert_eq!(count(&queue, "SELECT count(*) FROM completion_receipts"), 0);
    stop(&queue, second);
}

#[test]
fn runtime_bootstrap_retries_pending_delivery_with_original_id_without_queue_rpc() {
    let queue = Queue::new();
    let path = clock(&queue, 0);
    let first = owner(&queue, &path);
    queue.topic("cpu");
    let log = queue.log("retry");
    queue.call(&[
        "plugin",
        "enable",
        "retry",
        env!("CARGO_BIN_EXE_anyq-plugin-fixture"),
        "crash-once",
        log.to_str().unwrap(),
    ]);
    wait(|| count(&queue, "SELECT running FROM plugins WHERE name='retry'") == 1);
    queue.task("owner", &["cpu"]);
    wait_for_file(&log.with_extension("marker"), Some("crashed"));
    wait(|| count(&queue, "SELECT failures FROM plugins WHERE name='retry'") == 1);
    let cursor = count(&queue, "SELECT cursor FROM plugins WHERE name='retry'");
    let event = count(&queue, "SELECT max(sequence) FROM queue_events");
    assert!(cursor < event);
    stop(&queue, first);
    clock(&queue, 100_000);

    let second = owner(&queue, &path);
    wait(|| count(&queue, "SELECT cursor FROM plugins WHERE name='retry'") == event);
    let deliveries = fs::read_to_string(&log)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .collect::<Vec<_>>();

    assert_eq!(deliveries.len(), 2);
    assert_eq!(deliveries[0]["id"], deliveries[1]["id"]);
    assert_eq!(
        count(&queue, "SELECT max(sequence) FROM queue_events"),
        event
    );
    assert_eq!(queue.state("owner"), "running");
    stop(&queue, second);
}

#[test]
fn runtime_bootstrap_retains_uncertain_work_and_blocks_overdue_completion() {
    let queue = Queue::new();
    let path = clock(&queue, 0);
    let first = owner(&queue, &path);
    let log = queue.log("worker");
    queue.call(&[
        "plugin",
        "enable",
        "worker",
        env!("CARGO_BIN_EXE_anyq-plugin-fixture"),
        "uncertain",
        log.to_str().unwrap(),
    ]);
    wait(|| count(&queue, "SELECT running FROM plugins WHERE name='worker'") == 1);
    timeout(&queue);
    wait_for_file(&log.with_extension("marker"), Some("started"));
    assert!(!queue.raw(&["task", "complete", "owner"]).status.success());
    queue.call(&["plugin", "disable", "worker"]);
    wait(|| {
        count(
            &queue,
            "SELECT count(*) FROM managed_work WHERE state='uncertain'",
        ) == 1
    });
    stop(&queue, first);
    let original_work: String = Connection::open(&queue.db)
        .unwrap()
        .query_row("SELECT details FROM managed_work", [], |r| r.get(0))
        .unwrap();
    let launches = fs::read_to_string(&log).unwrap();
    clock(&queue, 11);

    let second = owner(&queue, &path);
    wait(|| {
        let state: String = Connection::open(&queue.db)
            .unwrap()
            .query_row("SELECT state FROM plugins WHERE name='timeout'", [], |r| {
                r.get(0)
            })
            .unwrap();
        state.contains("last_error")
    });

    assert_eq!(queue.state("owner"), "running");
    assert_eq!(queue.state("next"), "queued");
    assert_eq!(queue.usage("cpu"), 1);
    assert_eq!(
        count(
            &queue,
            "SELECT count(*) FROM managed_work WHERE state='uncertain'"
        ),
        1
    );
    assert_eq!(
        Connection::open(&queue.db)
            .unwrap()
            .query_row("SELECT details FROM managed_work", [], |r| r
                .get::<_, String>(0))
            .unwrap(),
        original_work
    );
    assert_eq!(fs::read_to_string(&log).unwrap(), launches);
    assert_eq!(count(&queue, "SELECT count(*) FROM completion_receipts"), 0);
    stop(&queue, second);
}

#[test]
fn runtime_bootstrap_keeps_owner_alive_without_creating_or_upgrading_database() {
    for version in [-4, -3, -2, -1, 0, 2, 3, 4, 5, 6, 99] {
        let queue = Queue::new();
        let path = clock(&queue, 0);
        if version == -3 {
            fs::write(&queue.db, b"not a sqlite database").unwrap();
        } else if version == -1 {
            fs::write(&queue.db, []).unwrap();
        } else if version == 6 || version == -4 {
            queue.topic("cpu");
            queue.call(&["daemon", "stop"]);
            let connection = Connection::open(&queue.db).unwrap();
            connection
                .execute_batch(if version == 6 {
                    "DROP TABLE plugin_timers"
                } else {
                    "ALTER TABLE managed_work DROP COLUMN details"
                })
                .unwrap();
        } else if version >= 0 {
            let connection = Connection::open(&queue.db).unwrap();
            connection.execute_batch("CREATE TABLE preserved(value TEXT); INSERT INTO preserved VALUES ('original');").unwrap();
            connection
                .pragma_update(None, "user_version", version)
                .unwrap();
            connection
                .pragma_update(None, "journal_mode", "WAL")
                .unwrap();
        }
        let before = fs::read(&queue.db).ok();

        let mut process = owner(&queue, &path);
        // Give bootstrap a real scheduling interval; status itself never opens SQLite.
        thread::sleep(Duration::from_millis(150));
        process.assert_running();
        assert!(
            queue
                .call(&["daemon", "status"])
                .starts_with("daemon is running (pid ")
        );
        stop(&queue, process);

        assert_eq!(fs::read(&queue.db).ok(), before, "schema {version}");
        for suffix in ["-wal", "-shm", "-journal"] {
            let mut sidecar = queue.db.as_os_str().to_owned();
            sidecar.push(suffix);
            assert!(!Path::new(&sidecar).exists(), "schema {version}: {suffix}");
        }
    }
}
