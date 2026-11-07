mod support;

use rusqlite::Connection;
use std::{
    fs,
    process::{Command, Stdio},
    thread,
    time::Duration,
};
use support::{
    process::TestProcess,
    queue::{Queue, wait_for_file},
};

#[test]
fn native_lifecycle_matches_the_user_protocol() {
    let q = Queue::new();

    for topic in [
        "itzdesign-commands",
        "pixel-10-emulator",
        "itzdesign-repo-lock",
    ] {
        q.topic(topic);
    }

    assert_eq!(
        q.call(&["topic", "--list"]),
        "- itzdesign-commands\n- itzdesign-repo-lock\n- pixel-10-emulator"
    );
    assert_eq!(
        q.call(&["task", "add", "my-task"]),
        "successfully registered task 'my-task'"
    );
    assert_eq!(q.state("my-task"), "registered");

    let unqueued = q.raw(&["task", "wait", "my-task"]);
    let unstarted = q.raw(&["task", "complete", "my-task"]);

    assert_eq!(unqueued.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&unqueued.stderr)
            .contains("'my-task' is not registered in any topic")
    );
    assert_eq!(unstarted.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&unstarted.stderr).contains("'my-task' was not run"));

    for topic in ["itzdesign-commands", "itzdesign-repo-lock"] {
        assert_eq!(
            q.call(&["topic", topic, "add", "my-task"]),
            format!("successfully added 'my-task' to topic '{topic}'")
        );
    }

    q.call(&["topic", "itzdesign-commands", "add", "my-task"]);
    assert_eq!(q.usage("itzdesign-commands"), 0);
    assert_eq!(q.call(&["task", "wait", "my-task"]), "'my-task' started");
    assert_eq!(q.state("my-task"), "running");
    assert_eq!(q.usage("itzdesign-commands"), 1);
    assert_eq!(q.usage("itzdesign-repo-lock"), 1);
    assert!(!q.raw(&["task", "wait", "my-task"]).status.success());
    assert!(
        !q.raw(&["topic", "pixel-10-emulator", "add", "my-task"])
            .status
            .success()
    );

    assert_eq!(
        q.call(&["task", "complete", "my-task"]),
        "'my-task' successfully completed"
    );

    assert_eq!(q.state("my-task"), "completed");
    assert_eq!(q.usage("itzdesign-commands"), 0);
    assert_eq!(q.usage("itzdesign-repo-lock"), 0);
    assert!(!q.raw(&["task", "complete", "my-task"]).status.success());
    assert!(!q.raw(&["task", "add", "my-task"]).status.success());
    assert!(
        !q.raw(&["task", "complete", "--force", "unknown"])
            .status
            .success()
    );
    assert!(
        !q.raw(&["topic", "unknown", "add", "my-task"])
            .status
            .success()
    );
}

#[test]
fn force_completes_registered_and_queued_tasks_without_granting_admission() {
    let q = Queue::new();
    q.topic("cpu");
    q.task("registered", &[]);
    q.task("queued", &["cpu"]);

    for name in ["registered", "queued"] {
        q.call(&["task", "complete", "--force", name]);

        assert_eq!(q.state(name), "completed");
        assert!(!q.raw(&["task", "wait", name]).status.success());
    }

    assert_eq!(q.usage("cpu"), 0);
}

#[test]
fn wait_for_two_topics_reserves_nothing_until_both_are_available() {
    let q = Queue::new();
    q.topic("cpu");
    q.topic("repo");
    q.task("owner", &["repo"]);
    q.call(&["task", "wait", "owner"]);
    q.task("build", &["cpu", "repo"]);

    let mut waiting = q.spawn_wait("build", "waiting");
    q.wait_for_log("waiting", "waiting for 'build' to start");

    waiting.assert_running();
    assert_eq!(q.state("build"), "queued");
    assert_eq!(q.usage("cpu"), 0);
    assert_eq!(q.usage("repo"), 1);

    q.call(&["task", "complete", "owner"]);
    let output = waiting.output();

    assert!(output.status.success());
    assert_eq!(q.state("build"), "running");
    assert_eq!(q.usage("cpu"), 1);
    assert_eq!(q.usage("repo"), 1);

    q.call(&["task", "complete", "build"]);

    assert_eq!(q.usage("cpu"), 0);
    assert_eq!(q.usage("repo"), 0);
}

#[test]
fn forced_completion_while_waiting_returns_error_instead_of_admission() {
    let q = Queue::new();
    q.topic("cpu");
    q.task("owner", &["cpu"]);
    q.call(&["task", "wait", "owner"]);
    q.task("pending", &["cpu"]);
    let mut pending = q.spawn_wait("pending", "pending");
    q.wait_for_log("pending", "waiting for 'pending'");

    pending.assert_running();
    q.call(&["task", "complete", "--force", "pending"]);
    let output = pending.output();

    assert_eq!(output.status.code(), Some(2));
    assert!(
        fs::read_to_string(q.log("pending"))
            .unwrap()
            .contains("already completed")
    );
    assert_eq!(q.state("pending"), "completed");
    assert_eq!(q.state("owner"), "running");
    assert_eq!(q.usage("cpu"), 1);
}

#[test]
fn two_processes_waiting_for_one_name_only_receive_one_permission() {
    let q = Queue::new();
    q.topic("cpu");
    q.task("build", &["cpu"]);

    let first = q.spawn_wait("build", "first");
    let second = q.spawn_wait("build", "second");
    let mut codes = [first.output().status.code(), second.output().status.code()];
    codes.sort();

    assert_eq!(codes, [Some(0), Some(2)]);
    assert_eq!(q.state("build"), "running");
    assert_eq!(q.usage("cpu"), 1);
}

#[test]
fn crossing_topic_links_use_global_task_order_across_processes() {
    let q = Queue::new();
    q.topic("cpu");
    q.topic("repo");
    q.task("first", &["cpu"]);
    q.task("second", &["repo"]);
    q.call(&["topic", "repo", "add", "first"]);
    q.call(&["topic", "cpu", "add", "second"]);
    let mut second = q.spawn_wait("second", "second");
    q.wait_for_log("second", "waiting for 'second'");

    q.call(&["task", "wait", "first"]);

    second.assert_running();
    assert_eq!(q.usage("cpu"), 1);
    assert_eq!(q.usage("repo"), 1);

    q.call(&["task", "complete", "first"]);

    assert!(second.output().status.success());
    assert_eq!(q.state("second"), "running");
    assert_eq!(q.usage("cpu"), 1);
    assert_eq!(q.usage("repo"), 1);
}

#[test]
fn capacity_two_is_respected_by_independent_wait_processes() {
    let q = Queue::new();
    q.call(&["topic", "create", "cpu", "--capacity", "2"]);

    for name in ["a", "b", "c"] {
        q.task(name, &["cpu"]);
    }

    let mut third = q.spawn_wait("c", "third");
    q.wait_for_log("third", "waiting for 'c'");
    let second = q.spawn_wait("b", "second");
    let first = q.spawn_wait("a", "first");

    assert!(second.output().status.success());
    assert!(first.output().status.success());
    third.assert_running();
    assert_eq!(q.usage("cpu"), 2);
    assert!(
        !q.raw(&["topic", "capacity", "cpu", "--capacity", "1"])
            .status
            .success()
    );

    q.call(&["task", "complete", "a"]);

    assert!(third.output().status.success());
    assert_eq!(q.usage("cpu"), 2);
}

#[test]
fn external_caller_fixture() {
    if std::env::var_os("ANYQ_TEST_CALLER").is_none() {
        return;
    }

    let status = Command::new(env!("CARGO_BIN_EXE_anyq"))
        .args(["task", "wait", "owner"])
        .stdout(Stdio::null())
        .status()
        .unwrap();

    assert!(status.success());

    fs::write(std::env::var_os("ANYQ_TEST_READY").unwrap(), "admitted").unwrap();
    thread::sleep(Duration::from_secs(60));
}

#[test]
fn killing_external_caller_and_waiter_does_not_release_or_complete_tasks() {
    let q = Queue::new();
    q.topic("cpu");
    q.task("owner", &["cpu"]);
    q.task("next", &["cpu"]);
    let ready = q.dir.path().join("ready");
    let mut caller = TestProcess::spawn(
        Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "external_caller_fixture", "--nocapture"])
            .env("ANYQ_TEST_CALLER", "1")
            .env("ANYQ_TEST_READY", &ready)
            .env("ANYQ_DB", &q.db)
            .stdout(Stdio::null())
            .stderr(Stdio::null()),
    );
    wait_for_file(&ready, Some("admitted"));
    let mut waiting = q.spawn_wait("next", "next");
    q.wait_for_log("next", "waiting for 'next'");

    caller.assert_running();
    caller.stop();

    assert_eq!(q.state("owner"), "running");
    assert_eq!(q.usage("cpu"), 1);
    waiting.assert_running();
    waiting.stop();

    assert_eq!(q.state("next"), "queued");
    q.call(&["task", "complete", "owner"]);
    q.call(&["task", "wait", "next"]);

    assert_eq!(q.state("next"), "running");
}

#[test]
fn adding_a_topic_racing_wait_is_atomic() {
    let q = Queue::new();
    q.topic("cpu");
    q.topic("repo");
    q.task("build", &["cpu"]);

    let addition = TestProcess::spawn(
        q.cmd(&["topic", "repo", "add", "build"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped()),
    );
    let admission = q.spawn_wait("build", "admission");
    let added = addition.output();

    assert!(admission.output().status.success());
    assert_eq!(q.state("build"), "running");
    assert_eq!(q.usage("cpu"), 1);
    assert_eq!(q.usage("repo"), i64::from(added.status.success()));

    if !added.status.success() {
        assert!(String::from_utf8_lossy(&added.stderr).contains("already running"));
    }
}

#[test]
fn force_racing_wait_always_leaves_no_owned_slots() {
    let q = Queue::new();
    q.topic("cpu");
    q.task("build", &["cpu"]);

    let force = TestProcess::spawn(
        q.cmd(&["task", "complete", "--force", "build"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped()),
    );
    let admission = q.spawn_wait("build", "admission");

    assert!(force.output().status.success());

    let output = admission.output();

    assert!(matches!(output.status.code(), Some(0 | 2)));
    assert_eq!(q.state("build"), "completed");
    assert_eq!(q.usage("cpu"), 0);

    if !output.status.success() {
        assert!(
            fs::read_to_string(q.log("admission"))
                .unwrap()
                .contains("already completed")
        );
    }
}

#[test]
fn reserved_topic_names_and_capacity_equals_syntax_are_unambiguous() {
    let q = Queue::new();
    q.topic("create");
    q.topic("capacity");
    q.call(&["topic", "create", "add", "--capacity=2"]);
    q.task("wait", &["create", "capacity", "add"]);

    q.call(&["topic", "capacity", "add", "--capacity=3"]);
    q.call(&["task", "wait", "wait"]);

    assert_eq!(q.state("wait"), "running");
    assert_eq!(q.usage("create"), 1);
    assert_eq!(q.usage("capacity"), 1);
    assert_eq!(q.usage("add"), 1);
}

#[test]
fn help_and_syntax_errors_do_not_open_a_database_and_explicit_path_wins() {
    let q = Queue::new();
    let help = q.raw(&["topic", "create", "--help"]);

    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("anyq topic create"));
    assert!(!q.db.exists());
    assert!(
        !q.raw(&["topic", "create", "cpu", "--capacity", "invalid"])
            .status
            .success()
    );
    assert!(!q.db.exists());

    let explicit = q.dir.path().join("explicit").join("queue.db");
    let result = TestProcess::spawn(
        q.cmd(&["topic", "create", "cpu", "--capacity", "2"])
            .arg("--db")
            .arg(&explicit)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped()),
    )
    .output();

    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(explicit.exists());
    assert!(!q.db.exists());
    assert_eq!(q.call(&["topic", "--list"]), "");
}

#[test]
fn the_legacy_cli_is_removed() {
    let q = Queue::new();

    for command in [
        "publish", "claim", "ack", "nack", "renew", "retry", "jobs", "group", "worker",
    ] {
        let output = q.raw(&[command]);

        assert_eq!(output.status.code(), Some(2), "{command}");
        assert!(!q.db.exists());
    }
}

#[test]
fn legacy_database_returns_a_clear_error_without_modifying_it() {
    let q = Queue::new();
    let connection = Connection::open(&q.db).unwrap();
    connection.execute_batch("CREATE TABLE messages(payload TEXT); INSERT INTO messages VALUES ('old-data'); PRAGMA user_version=1").unwrap();
    drop(connection);
    let before = fs::read(&q.db).unwrap();

    let result = q.raw(&["topic", "--list"]);

    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr).contains("separate database with --db"));
    assert_eq!(fs::read(&q.db).unwrap(), before);
}

#[cfg(windows)]
#[test]
fn windows_default_uses_local_app_data_and_anyq_db_overrides_it() {
    let q = Queue::new();
    let local = q.dir.path().join("local");
    let result = TestProcess::spawn(
        Command::new(env!("CARGO_BIN_EXE_anyq"))
            .args(["topic", "--list"])
            .env_remove("ANYQ_DB")
            .env("LOCALAPPDATA", &local)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped()),
    )
    .output();

    assert!(result.status.success());
    assert!(local.join("anyq/queue.db").exists());

    let result = q
        .cmd(&["topic", "--list"])
        .env_remove("LOCALAPPDATA")
        .output()
        .unwrap();

    assert!(result.status.success());
    assert!(q.db.exists());
}

#[cfg(unix)]
#[test]
fn unix_default_uses_xdg_then_home_without_touching_real_user_data() {
    let q = Queue::new();
    let xdg = q.dir.path().join("xdg");
    let home = q.dir.path().join("home");
    let result = TestProcess::spawn(
        Command::new(env!("CARGO_BIN_EXE_anyq"))
            .args(["topic", "--list"])
            .env_remove("ANYQ_DB")
            .env("XDG_DATA_HOME", &xdg)
            .env("HOME", &home)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped()),
    )
    .output();

    assert!(result.status.success());
    assert!(xdg.join("anyq/queue.db").exists());
    assert!(!home.exists());

    let result = TestProcess::spawn(
        Command::new(env!("CARGO_BIN_EXE_anyq"))
            .args(["topic", "--list"])
            .env_remove("ANYQ_DB")
            .env_remove("XDG_DATA_HOME")
            .env("HOME", &home)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped()),
    )
    .output();

    assert!(result.status.success());
    assert!(home.join(".local/share/anyq/queue.db").exists());
}
