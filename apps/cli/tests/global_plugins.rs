pub mod support;

#[path = "support/daemon_protocol.rs"]
pub mod daemon_protocol;

#[path = "support/eventually.rs"]
pub mod eventually;

use anyq_queue_api::{Metadata, MetadataValue};
use eventually::eventually as wait;
use serde_json::{Value, json};
use std::{fs, path::Path};
use support::queue::Queue;

fn call(queue: &Queue, database: &Path, args: &[&str]) -> String {
    let mut all = vec!["--db", database.to_str().unwrap()];
    all.extend_from_slice(args);

    queue.call(&all)
}

fn status(queue: &Queue, database: &Path, name: &str) -> Value {
    serde_json::from_str(&call(queue, database, &["plugin", "status", name])).unwrap()
}

fn inspect(queue: &Queue, database: &Path) -> Vec<Value> {
    serde_json::from_str(&call(queue, database, &["plugin", "inspect", "shell"])).unwrap()
}

fn file(queue: &Queue, name: &str, value: Value) -> std::path::PathBuf {
    let path = queue.dir.path().join(name);
    fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();

    path
}

#[test]
fn globally_enabled_standalone_shell_executes_equal_names_in_independent_queues_once() {
    let queue = Queue::new();
    queue.call(&["plugin", "list"]);
    assert!(!queue.db.exists());
    let second = queue.dir.path().join("second.db");

    for database in [&queue.db, &second] {
        call(&queue, database, &["daemon", "start"]);
        call(&queue, database, &["topic", "create", "cpu"]);
    }
    queue.call(&["plugin", "enable", "shell"]);
    for database in [&queue.db, &second] {
        wait(|| status(&queue, database, "shell")["running"] == true);
    }
    for (database, label) in [(&queue.db, "first"), (&second, "second")] {
        let metadata = file(
            &queue,
            &format!("{label}.json"),
            json!({
                "shell": {"sh":[format!("echo {label} >> {label}.log")], "cwd":queue.dir.path(), "autocomplete":true},
                "third_party": {"nested":[null, true, {"arbitrary":"kept"}]}
            }),
        );
        call(
            &queue,
            database,
            &[
                "task",
                "create",
                "build",
                "-t",
                "cpu",
                "--metafile",
                metadata.to_str().unwrap(),
            ],
        );
    }
    for database in [&queue.db, &second] {
        wait(|| call(&queue, database, &["task", "build"]).contains("status: completed"));
    }

    for (database, label) in [(&queue.db, "first"), (&second, "second")] {
        let work = inspect(&queue, database);
        assert_eq!(work.len(), 1);
        assert_eq!(work[0]["state"], "stopped");
        let dto: anyq_queue_wire::MetadataDto =
            serde_json::from_value(work[0]["details"].clone()).unwrap();
        let details: Metadata = dto.try_into().unwrap();
        assert_eq!(
            details.get(Some("status")).unwrap(),
            &MetadataValue::String("exited".into())
        );
        let saved: Value = serde_json::from_str(&call(
            &queue,
            database,
            &["task", "build", "meta", "third_party"],
        ))
        .unwrap();
        assert_eq!(saved, json!({"nested":[null, true, {"arbitrary":"kept"}]}));
        assert_eq!(
            fs::read_to_string(queue.dir.path().join(format!("{label}.log")))
                .unwrap()
                .trim(),
            label
        );
        call(&queue, database, &["daemon", "stop"]);
        call(&queue, database, &["daemon", "start"]);
        wait(|| status(&queue, database, "shell")["running"] == true);

        assert_eq!(inspect(&queue, database), work);
        assert_eq!(
            fs::read_to_string(queue.dir.path().join(format!("{label}.log")))
                .unwrap()
                .lines()
                .count(),
            1
        );
    }
}

#[test]
fn global_disable_and_restart_preserve_each_queues_original_timeout_and_state() {
    let queue = Queue::new();
    let second = queue.dir.path().join("second.db");
    let stopped = queue.dir.path().join("stopped.db");
    call(&queue, &stopped, &["plugin", "enable", "timeout"]);
    assert!(!stopped.exists());

    let mut deadlines = Vec::new();
    for database in [&queue.db, &second] {
        call(&queue, database, &["daemon", "start"]);
        call(
            &queue,
            database,
            &["topic", "create", "cpu", "--meta", "timeout", "\"1d\""],
        );
        wait(|| status(&queue, database, "timeout")["running"] == true);
        call(&queue, database, &["task", "create", "build", "-t", "cpu"]);
        wait(|| status(&queue, database, "timeout")["active_timers"] == 1);
        let connection = rusqlite::Connection::open(database).unwrap();
        let deadline: i64 = connection
            .query_row(
                "SELECT deadline FROM plugin_timers WHERE plugin='timeout'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        deadlines.push(deadline);
    }

    queue.call(&["plugin", "disable", "timeout"]);
    for database in [&queue.db, &second] {
        wait(|| status(&queue, database, "timeout")["running"] == false);
        assert_eq!(status(&queue, database, "timeout")["active_timers"], 1);
        call(&queue, database, &["daemon", "stop"]);
        call(&queue, database, &["daemon", "start"]);
        assert_eq!(status(&queue, database, "timeout")["running"], false);
    }
    queue.call(&["plugin", "enable", "timeout"]);
    for (index, database) in [&queue.db, &second].into_iter().enumerate() {
        wait(|| status(&queue, database, "timeout")["running"] == true);
        let connection = rusqlite::Connection::open(database).unwrap();
        let restored: i64 = connection
            .query_row(
                "SELECT deadline FROM plugin_timers WHERE plugin='timeout'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(restored, deadlines[index]);
    }

    queue.call(&["task", "complete", "build"]);
    wait(|| status(&queue, &queue.db, "timeout")["active_timers"] == 0);

    assert!(call(&queue, &second, &["task", "build"]).contains("status: running"));
    assert_eq!(status(&queue, &second, "timeout")["active_timers"], 1);
    assert!(!stopped.exists());
}

#[test]
fn standalone_timeout_completes_a_metadata_deadline_after_admission() {
    let queue = Queue::new();
    queue.call(&["plugin", "enable", "timeout"]);
    queue.topic("cpu");
    wait(|| status(&queue, &queue.db, "timeout")["running"] == true);
    let metadata = file(
        &queue,
        "deadline.json",
        json!({"timeout":"1ms", "unrelated":{"opaque":true}}),
    );

    queue.call(&[
        "task",
        "create",
        "deadline",
        "-t",
        "cpu",
        "--metafile",
        metadata.to_str().unwrap(),
    ]);
    wait(|| queue.state("deadline") == "completed");
    wait(|| status(&queue, &queue.db, "timeout")["active_timers"] == 0);

    assert_eq!(queue.usage("cpu"), 0);
    assert_eq!(status(&queue, &queue.db, "timeout")["active_timers"], 0);
}

#[test]
fn legacy_wire_enablement_cannot_bypass_the_common_catalog() {
    use anyq_daemon_api::{QueueRequest, QueueResponse};
    use daemon_protocol::{open, request};

    let queue = Queue::new();
    queue.call(&["daemon", "start"]);
    let endpoint = anyq_daemon_transport::Endpoint::resolve(&queue.db).unwrap();
    let stream = open(&endpoint);

    let enabled = request(
        &stream,
        2,
        QueueRequest::PluginEnable {
            name: "rogue".into(),
            executable: env!("CARGO_BIN_EXE_anyq").into(),
            arguments: vec![],
        },
    );
    let disabled = request(
        &stream,
        3,
        QueueRequest::PluginDisable {
            name: "shell".into(),
        },
    );
    let listed = request(&stream, 4, QueueRequest::PluginList);

    assert!(
        matches!(enabled, QueueResponse::Error(anyq_daemon_api::QueueError::Invalid(message)) if message.contains("user-wide"))
    );
    assert!(
        matches!(disabled, QueueResponse::Error(anyq_daemon_api::QueueError::Invalid(message)) if message.contains("user-wide"))
    );
    assert!(matches!(listed, QueueResponse::Plugins(plugins) if plugins.is_empty()));
}

#[test]
fn invalid_catalog_and_unavailable_binary_stop_only_plugins_and_preserve_each_timer() {
    let queue = Queue::new();
    let second = queue.dir.path().join("second.db");
    queue.call(&["plugin", "enable", "timeout"]);
    let mut owners = Vec::new();
    for database in [&queue.db, &second] {
        call(&queue, database, &["daemon", "start"]);
        owners.push(call(&queue, database, &["daemon", "status"]));
        call(
            &queue,
            database,
            &["topic", "create", "cpu", "--meta", "timeout", "\"1d\""],
        );
        wait(|| status(&queue, database, "timeout")["running"] == true);
        call(&queue, database, &["task", "create", "build", "-t", "cpu"]);
        wait(|| status(&queue, database, "timeout")["active_timers"] == 1);
    }
    let catalog_path = queue.dir.path().join(".anyq/plugins/.catalog.toml");
    let saved = fs::read(&catalog_path).unwrap();

    fs::write(&catalog_path, b"broken = [").unwrap();
    for (index, database) in [&queue.db, &second].into_iter().enumerate() {
        wait(|| status(&queue, database, "timeout")["running"] == false);
        assert_eq!(status(&queue, database, "timeout")["active_timers"], 1);
        assert!(
            status(&queue, database, "timeout")["last_failure"]
                .as_str()
                .unwrap()
                .contains("catalog")
        );
        assert_eq!(call(&queue, database, &["daemon", "status"]), owners[index]);
        assert!(call(&queue, database, &["task", "build"]).contains("status: running"));
    }

    fs::write(&catalog_path, &saved).unwrap();
    for database in [&queue.db, &second] {
        wait(|| status(&queue, database, "timeout")["running"] == true);
    }
    let executable = queue.dir.path().join(format!(
        ".anyq/plugins/timeout{}",
        std::env::consts::EXE_SUFFIX
    ));
    queue.call(&["plugin", "disable", "timeout"]);
    for database in [&queue.db, &second] {
        wait(|| status(&queue, database, "timeout")["running"] == false);
    }
    fs::remove_file(&executable).unwrap();
    fs::create_dir(&executable).unwrap();
    fs::write(&catalog_path, &saved).unwrap();
    for database in [&queue.db, &second] {
        wait(|| {
            status(&queue, database, "timeout")["last_failure"]
                .as_str()
                .is_some_and(|message| message.contains("unavailable"))
        });
        assert_eq!(status(&queue, database, "timeout")["running"], false);
        assert_eq!(status(&queue, database, "timeout")["active_timers"], 1);
        assert!(call(&queue, database, &["task", "build"]).contains("status: running"));
    }
    fs::remove_dir(&executable).unwrap();
    fs::copy(env!("CARGO_BIN_EXE_anyq-timeout"), &executable).unwrap();
    queue.call(&["plugin", "enable", "timeout"]);
    for database in [&queue.db, &second] {
        wait(|| status(&queue, database, "timeout")["running"] == true);
        assert_eq!(status(&queue, database, "timeout")["active_timers"], 1);
    }
}
