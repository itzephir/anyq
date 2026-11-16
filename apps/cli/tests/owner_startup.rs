pub mod support;

#[path = "support/eventually.rs"]
pub mod eventually;

use eventually::eventually;
use std::{fs, process::Stdio};
use support::{process::TestProcess, queue::Queue};

#[test]
fn default_commands_start_the_owner_but_custom_commands_do_not_create_storage() {
    let queue = Queue::new();
    let custom = queue.dir.path().join("absent/other.db");
    let path = custom.to_str().unwrap();

    let failed = queue.raw(&["--db", path, "task", "list"]);

    assert_eq!(failed.status.code(), Some(2));
    assert!(
        String::from_utf8(failed.stderr)
            .unwrap()
            .contains("daemon start")
    );
    assert!(!custom.parent().unwrap().exists());
    assert!(!queue.db.exists());
    assert_eq!(
        queue.call(&["--db", path, "daemon", "status"]),
        "daemon is stopped"
    );

    queue.topic("cpu");

    assert!(queue.call(&["daemon", "status"]).contains("running"));
    assert!(!custom.parent().unwrap().exists());
}

#[test]
fn explicit_custom_start_and_restart_preserve_ownership_without_implicit_restarts() {
    let queue = Queue::new();
    queue.topic("cpu");
    let original_default = queue.call(&["daemon", "status"]);
    let custom = queue.dir.path().join("custom/other.db");
    let path = custom.to_str().unwrap();

    queue.call(&["--db", path, "daemon", "start"]);
    queue.call(&["--db", path, "topic", "create", "cpu"]);
    queue.call(&["--db", path, "task", "create", "owner", "-t", "cpu"]);
    queue.call(&["--db", path, "task", "create", "next", "-t", "cpu"]);
    queue.call(&["--db", path, "daemon", "stop"]);
    let failed = queue.raw(&["--db", path, "task", "owner"]);

    assert_eq!(failed.status.code(), Some(2));
    assert_eq!(
        queue.call(&["--db", path, "daemon", "status"]),
        "daemon is stopped"
    );
    assert_eq!(queue.call(&["daemon", "status"]), original_default);

    queue.call(&["--db", path, "daemon", "start"]);

    assert!(
        queue
            .call(&["--db", path, "task", "owner"])
            .contains("status: running")
    );
    assert!(
        queue
            .call(&["--db", path, "task", "next"])
            .contains("status: queued")
    );
    queue.call(&["--db", path, "task", "complete", "owner"]);
    assert!(
        queue
            .call(&["--db", path, "task", "next"])
            .contains("status: running")
    );
}

#[test]
fn a_crashed_custom_owner_waits_for_an_explicit_restart() {
    let queue = Queue::new();
    let custom = queue.dir.path().join("other.db");
    let path = custom.to_str().unwrap();
    let mut command = queue.cmd(&["--db", path, "daemon", "run"]);
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let owner = TestProcess::spawn(&mut command);
    eventually(|| {
        queue
            .call(&["--db", path, "daemon", "status"])
            .contains("running")
    });
    queue.call(&["--db", path, "topic", "create", "cpu"]);
    queue.call(&["--db", path, "task", "create", "owner", "-t", "cpu"]);

    owner.stop();
    let failed = queue.raw(&["--db", path, "task", "owner"]);

    assert_eq!(failed.status.code(), Some(2));
    assert_eq!(
        queue.call(&["--db", path, "daemon", "status"]),
        "daemon is stopped"
    );

    queue.call(&["--db", path, "daemon", "start"]);

    assert!(
        queue
            .call(&["--db", path, "task", "owner"])
            .contains("status: running")
    );
}

#[test]
fn an_explicit_equivalent_default_path_keeps_automatic_startup() {
    let queue = Queue::new();
    queue.topic("cpu");
    queue.call(&["daemon", "stop"]);
    let alias = queue
        .db
        .parent()
        .unwrap()
        .join(".")
        .join(queue.db.file_name().unwrap());

    queue.call(&["--db", alias.to_str().unwrap(), "task", "list"]);

    assert!(queue.call(&["daemon", "status"]).contains("running"));
}

#[test]
fn shared_configuration_selects_the_automatic_default_when_environment_is_absent() {
    let queue = Queue::new();
    let configured = queue.dir.path().join("configured.db");
    fs::create_dir_all(queue.dir.path().join(".anyq")).unwrap();
    fs::write(
        queue.dir.path().join(".anyq/config.toml"),
        format!(
            "[db]\npath = {}\n",
            serde_json::to_string(&configured).unwrap()
        ),
    )
    .unwrap();
    let mut command = queue.cmd(&["task", "list"]);
    command
        .env_remove("ANYQ_DB")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let output = TestProcess::spawn(&mut command).output();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(configured.is_file());
    assert!(!queue.db.exists());
    assert!(
        queue
            .call(&["--db", configured.to_str().unwrap(), "daemon", "status"])
            .contains("running")
    );
}
