use crate::support::{process::TestProcess, queue::Queue};
use std::{fs, process::Stdio};

#[test]
fn global_autostart_defaults_to_true_and_changes_new_tasks_without_queue_io() {
    let q = Queue::new();
    assert_eq!(q.call(&["config", "task.autostart"]), "true");
    assert!(!q.db.exists());
    q.call(&["config", "task.autostart", "false"]);
    assert!(!q.db.exists());
    q.topic("cpu");
    q.task("manual", &["cpu"]);

    assert_eq!(q.state("manual"), "queued");
    assert_eq!(q.usage("cpu"), 0);
    assert_eq!(q.call(&["task", "manual", "start"]), "'manual' started");
    assert_eq!(
        q.call(&["task", "start", "manual"]),
        "'manual' is already running"
    );
    assert_eq!(q.state("manual"), "running");
    q.call(&["task", "complete", "manual"]);
    q.call(&["config", "--unset", "task.autostart"]);
    assert_eq!(q.call(&["config", "task.autostart"]), "true");
    q.task("automatic", &["cpu"]);
    assert_eq!(q.state("automatic"), "running");
}

#[test]
fn topic_overrides_global_setting_and_capacity_updates_preserve_the_override() {
    for (global, topic, expected) in [(false, "on", "running"), (true, "off", "queued")] {
        let q = Queue::new();
        q.call(&[
            "config",
            "task.autostart",
            if global { "true" } else { "false" },
        ]);
        q.topic("cpu");
        q.call(&["topic", "cpu", "autostart", topic]);
        q.call(&["topic", "capacity", "cpu", "2"]);
        q.call(&["topic", "create", "cpu", "-c", "2"]);

        q.task("job", &["cpu"]);

        assert_eq!(q.state("job"), expected);
        if expected == "queued" {
            q.call(&["topic", "autostart", "cpu"]);
            assert_eq!(q.state("job"), "running");
        }
        assert_eq!(q.usage("cpu"), 1);
    }
}

#[test]
fn one_disabled_topic_blocks_automatic_start_but_explicit_start_acquires_every_slot() {
    let q = Queue::new();
    q.topic("cpu");
    q.topic("repo");
    q.call(&["topic", "repo", "autostart", "off"]);
    q.task("job", &["cpu", "repo"]);

    assert_eq!(q.state("job"), "queued");
    assert_eq!(q.usage("cpu"), 0);
    assert_eq!(q.usage("repo"), 0);

    q.call(&["task", "start", "job"]);

    assert_eq!(q.state("job"), "running");
    assert_eq!(q.usage("cpu"), 1);
    assert_eq!(q.usage("repo"), 1);
}

#[test]
fn stop_and_short_toggle_persist_until_explicit_start_despite_queue_changes() {
    for flag in ["--stop", "-s"] {
        let q = Queue::new();
        q.topic("cpu");
        q.call(&["task", "create", "job", "-t", "cpu", flag]);
        q.task("later", &["cpu"]);
        q.call(&["topic", "capacity", "cpu", "2"]);
        q.call(&["topic", "cpu", "autostart"]);
        q.call(&["task", "complete", "later", "-f"]);

        assert_eq!(q.state("job"), "queued");
        assert_eq!(q.usage("cpu"), 0);
        q.call(&["task", "job", "start"]);
        assert_eq!(q.state("job"), "running");
    }
}

#[test]
fn explicit_creation_start_and_toggle_work_when_autostart_is_disabled() {
    for flag in ["--start", "-s"] {
        let q = Queue::new();
        q.topic("cpu");
        q.call(&["config", "task.autostart", "false"]);

        q.call(&["task", "create", "job", "-t", "cpu", flag]);

        assert_eq!(q.state("job"), "running");
        assert_eq!(q.usage("cpu"), 1);
    }
}

#[test]
fn explicit_start_fails_without_available_admission_and_never_reserves_partially() {
    let q = Queue::new();
    q.topic("cpu");
    q.topic("repo");
    q.task("owner", &["repo"]);
    q.call(&[
        "task", "create", "manual", "-t", "cpu", "-t", "repo", "--stop",
    ]);
    let before = q.call(&["task", "manual"]);

    let output = q.raw(&["task", "manual", "start"]);

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot start"));
    assert_eq!(q.call(&["task", "manual"]), before);
    assert_eq!(q.usage("cpu"), 0);
    assert_eq!(q.usage("repo"), 1);
    q.call(&["task", "complete", "owner"]);
    assert_eq!(q.state("manual"), "queued");
    q.call(&["task", "start", "manual"]);
    assert_eq!(q.state("manual"), "running");
}

#[test]
fn start_wait_complete_waits_for_all_resources_and_then_completes() {
    for command in [
        vec!["task", "manual", "start", "-wc"],
        vec!["task", "start", "manual", "--wait", "--complete"],
    ] {
        let q = Queue::new();
        q.topic("cpu");
        q.task("owner", &["cpu"]);
        q.call(&["task", "create", "manual", "-t", "cpu", "--stop"]);
        let mut child = TestProcess::spawn(
            q.cmd(&command)
                .stdout(Stdio::piped())
                .stderr(fs::File::create(q.log("start")).unwrap()),
        );
        q.wait_for_log("start", "waiting for 'manual'");
        child.assert_running();
        assert_eq!(q.state("manual"), "queued");

        q.call(&["task", "complete", "owner"]);
        let result = child.output();

        assert!(result.status.success());
        assert_eq!(
            String::from_utf8(result.stdout).unwrap(),
            "'manual' started\n'manual' successfully completed\n"
        );
        assert_eq!(q.state("manual"), "completed");
        assert_eq!(q.usage("cpu"), 0);
    }
}

#[test]
fn tswc_cluster_toggles_effective_autostart_and_preserves_the_chain() {
    for automatic in [false, true] {
        let q = Queue::new();
        q.topic("cpu");
        q.call(&[
            "config",
            "task.autostart",
            if automatic { "true" } else { "false" },
        ]);
        let child = TestProcess::spawn(
            q.cmd(&["task", "create", "job", "-tswc", "cpu"])
                .stdout(Stdio::piped())
                .stderr(fs::File::create(q.log("cluster")).unwrap()),
        );

        if automatic {
            q.wait_for_log("cluster", "waiting for 'job'");
            assert_eq!(q.state("job"), "queued");
            assert_eq!(q.usage("cpu"), 0);
            q.call(&["task", "job", "start"]);
        }

        assert!(child.output().status.success());
        assert_eq!(q.state("job"), "completed");
        assert_eq!(q.usage("cpu"), 0);
    }
}

#[test]
fn start_and_stop_syntax_errors_happen_before_database_access() {
    let q = Queue::new();
    for arguments in [
        vec!["task", "create", "job", "--start", "--stop"],
        vec!["task", "create", "job", "--start", "-s"],
        vec!["task", "create", "job", "--stop", "-s"],
        vec!["task", "job", "start", "-c"],
        vec!["topic", "cpu", "autostart", "invalid"],
        vec!["task", "link", "job"],
        vec!["topic", "link", "cpu"],
    ] {
        assert_eq!(q.raw(&arguments).status.code(), Some(2), "{arguments:?}");
        assert!(!q.db.exists());
    }
}
