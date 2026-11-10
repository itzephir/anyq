mod support;

#[path = "support/config_regressions.rs"]
mod config_regressions;

#[path = "support/output_regressions.rs"]
mod output_regressions;

#[path = "support/cli_extensions.rs"]
mod cli_extensions;

#[path = "support/automatic_admission.rs"]
mod automatic_admission;

#[path = "support/autostart.rs"]
mod autostart;

#[path = "support/observation.rs"]
mod observation;

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
fn bare_and_backtick_wrapped_names_share_registration_links_and_lookup() {
    let q = Queue::new();
    q.topic("`cpu`");
    q.topic("cpu");
    q.call(&["task", "create", "`build`", "-t", "`cpu`"]);

    assert_eq!(q.call(&["topic", "-l"]), "- cpu");
    assert_eq!(q.call(&["task", "-l"]), "- build");
    assert_eq!(q.raw(&["task", "create", "build"]).status.code(), Some(2));
    assert_eq!(
        q.raw(&["task", "link", "build", "cpu"]).status.code(),
        Some(2)
    );
    q.call(&["task", "wait", "`build`"]);

    assert_eq!(
        q.call(&["task", "build"]),
        "task: build\nstatus: running\ntopics:\n- cpu"
    );
    assert!(q.call(&["topic", "`cpu`"]).contains("- build [running]"));
    q.call(&["topic", "capacity", "`cpu`", "2"]);
    assert_eq!(q.call(&["topic", "capacity", "cpu"]), "2");
    q.call(&["task", "complete", "build"]);

    assert_eq!(q.state("build"), "completed");
    assert_eq!(q.usage("cpu"), 0);

    for name in ["`create`", "`capacity`", "`link`", "`help`"] {
        q.topic(name);
    }
}

#[test]
fn arbitrary_name_text_and_quoted_initials_work_through_chained_cli_and_storage() {
    let q = Queue::new();

    for (index, topic) in [
        "a spaces/символы #$%@!^&*+~?><,.;:(){}[]\"'",
        "`🔥 ресурс`",
        "`--leading`",
        "``literal``",
        "``",
    ]
    .into_iter()
    .enumerate()
    {
        let name = format!("`задача {index}`");
        q.topic(topic);
        let trace = q.call(&["task", "create", &name, "-twc", topic]);

        assert!(trace.contains(&format!("'{name}' started")));
        assert!(trace.contains(&format!("'{name}' successfully completed")));
        assert_eq!(q.state(&format!("задача {index}")), "completed");
        assert_eq!(
            q.call(&["task", &name]),
            format!("task: {name}\nstatus: completed\ntopics:\n- {topic}")
        );
        assert!(q.call(&["topic", topic]).ends_with("queue: (empty)"));
    }
}

#[test]
fn topic_and_task_inspection_show_state_capacity_and_only_current_queue_entries() {
    let q = Queue::new();
    q.topic("cpu");
    q.topic("other");

    assert_eq!(
        q.call(&["topic", "cpu"]),
        "topic: cpu\ncapacity: 1\nqueue: (empty)"
    );
    assert_eq!(q.call(&["topic", "capacity", "cpu"]), "1");
    q.task("z-running", &["cpu"]);
    q.call(&["task", "wait", "z-running"]);
    q.task("a-queued", &["cpu"]);
    q.task("completed", &["cpu"]);
    q.call(&["task", "complete", "-f", "completed"]);
    q.task("registered", &[]);
    q.task("unrelated", &["other"]);

    assert_eq!(
        q.call(&["topic", "cpu"]),
        "topic: cpu\ncapacity: 1\nqueue:\n- z-running [running]\n- a-queued [queued]"
    );
    assert_eq!(
        q.call(&["task", "a-queued"]),
        "task: a-queued\nstatus: queued\ntopics:\n- cpu"
    );
    assert_eq!(
        q.call(&["task", "registered"]),
        "task: registered\nstatus: registered\ntopics: (none)"
    );
    assert_eq!(
        q.call(&["task", "completed"]),
        "task: completed\nstatus: completed\ntopics:\n- cpu"
    );
    assert_eq!(
        q.call(&["topic", "capacity", "cpu", "2"]),
        "capacity of 'cpu' set to 2"
    );
    assert_eq!(q.call(&["topic", "capacity", "cpu"]), "2");
    assert_eq!(q.usage("cpu"), 2);
    assert_eq!(q.state("a-queued"), "running");

    for command in [
        vec!["topic", "missing"],
        vec!["task", "missing"],
        vec!["topic", "capacity", "missing"],
    ] {
        assert_eq!(q.raw(&command).status.code(), Some(2));
    }
}

#[test]
fn task_names_can_be_inspected_with_a_separator_and_global_database_option() {
    let q = Queue::new();
    q.topic("cpu");
    q.task("`add`", &["cpu"]);
    q.call(&["task", "create", "--", "`-twc`"]);

    assert_eq!(
        q.call(&["task", "--", "`-twc`"]),
        "task: `-twc`\nstatus: registered\ntopics: (none)"
    );

    assert_eq!(
        q.call(&["task", "--", "`add`"]),
        "task: `add`\nstatus: running\ntopics:\n- cpu"
    );

    let output = TestProcess::spawn(
        q.cmd(&["task", "--db"])
            .arg(&q.db)
            .arg("--")
            .arg("`add`")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped()),
    )
    .output();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("status: running"));
}

#[test]
fn both_link_forms_share_membership_idempotency_and_state_errors() {
    let q = Queue::new();
    q.topic("cpu");
    q.task("owner", &["cpu"]);
    q.task("build", &[]);
    let commands = [
        ["topic", "link", "cpu", "build"],
        ["task", "link", "build", "cpu"],
    ];

    for command in &commands {
        assert_eq!(q.call(command), "successfully added 'build' to topic 'cpu'");
    }

    assert_eq!(q.state("build"), "queued");
    assert_eq!(q.usage("cpu"), 1);
    q.call(&["task", "complete", "owner"]);
    q.call(&["task", "wait", "build"]);
    assert_eq!(q.usage("cpu"), 1);

    for command in &commands {
        let error = q.raw(command);

        assert_eq!(error.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&error.stderr).contains("already running"));
    }

    q.call(&["task", "complete", "build"]);

    for command in &commands {
        assert_eq!(q.raw(command).status.code(), Some(2));
    }

    for command in [
        ["topic", "link", "missing", "build"],
        ["task", "link", "missing", "cpu"],
    ] {
        assert_eq!(q.raw(&command).status.code(), Some(2));
    }
}

#[test]
fn short_options_and_initial_topics_preserve_capacity_and_manual_ownership() {
    let q = Queue::new();
    q.call(&["topic", "create", "cpu", "-c", "2"]);
    q.call(&["topic", "create", "repo", "--capacity=2"]);

    assert_eq!(q.call(&["topic", "-l"]), q.call(&["topic", "--list"]));
    assert_eq!(
        q.call(&["topic", "capacity", "cpu", "2"]),
        "capacity of 'cpu' set to 2"
    );

    for name in ["a", "b"] {
        assert_eq!(
            q.call(&[
                "task", "create", name, "--topic", "cpu", "-t", "repo", "-t", "cpu"
            ]),
            format!("successfully registered task '{name}'")
        );
        assert_eq!(q.state(name), "running");
    }

    assert_eq!(q.usage("cpu"), 2);
    assert_eq!(q.usage("repo"), 2);
    q.call(&["task", "wait", "a"]);
    q.call(&["task", "wait", "b"]);
    assert_eq!(q.usage("cpu"), 2);
    assert_eq!(q.usage("repo"), 2);

    q.call(&["task", "complete", "a", "-f"]);
    q.call(&["task", "complete", "--force", "b"]);

    assert_eq!(q.usage("cpu"), 0);
    assert_eq!(q.usage("repo"), 0);
}

#[test]
fn registration_with_unknown_initial_topic_is_atomic_and_can_be_retried() {
    let q = Queue::new();
    q.topic("cpu");
    q.topic("repo");

    let failed = q.raw(&["task", "create", "build", "-t", "cpu", "-t", "missing"]);

    assert_eq!(failed.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&failed.stderr).contains("topic 'missing' does not exist"));
    assert_eq!(q.call(&["task", "--list"]), "");

    q.call(&["task", "create", "build", "-t", "cpu", "--topic", "repo"]);
    q.call(&["task", "wait", "build"]);

    assert_eq!(q.state("build"), "running");
    assert_eq!(q.usage("cpu"), 1);
    assert_eq!(q.usage("repo"), 1);
}

#[test]
fn flag_dependencies_and_invalid_syntax_fail_before_opening_a_database() {
    let q = Queue::new();

    for command in [
        vec!["topic", "capacity", "cpu", "0"],
        vec!["topic", "--list", "create", "cpu"],
        vec!["task", "-l", "create", "build"],
        vec!["task", "create", "build", "--wait"],
        vec!["task", "create", "build", "--complete"],
        vec!["task", "create", "build", "--topic", "cpu", "--complete"],
        vec!["task", "create", "build", "-tc", "cpu"],
    ] {
        let output = q.raw(&command);

        assert_eq!(output.status.code(), Some(2), "{command:?}");
        assert!(!q.db.exists(), "{command:?}");
    }
}

#[test]
fn help_and_version_shortcuts_are_equal_and_do_not_open_a_database() {
    let q = Queue::new();

    assert_eq!(q.call(&["-v"]), q.call(&["--version"]));
    assert_eq!(q.call(&["-h"]), q.call(&["--help"]));
    let help = q.call(&["task", "create", "--help"]);

    for option in ["--topic", "--wait", "--complete"] {
        assert!(help.contains(option));
    }

    assert!(!q.db.exists());
}

#[test]
fn chained_registration_waits_before_completing_and_can_keep_ownership() {
    let q = Queue::new();
    q.topic("cpu");
    q.topic("repo");
    q.task("owner", &["cpu"]);
    q.call(&["task", "wait", "owner"]);
    let mut chained = TestProcess::spawn(
        q.cmd(&["task", "create", "chained", "-twc", "cpu", "-t", "repo"])
            .arg("--db")
            .arg(&q.db)
            .stdout(Stdio::piped())
            .stderr(fs::File::create(q.log("chained")).unwrap()),
    );
    q.wait_for_log("chained", "waiting for 'chained' to start");

    chained.assert_running();
    assert_eq!(q.state("chained"), "queued");
    assert_eq!(q.usage("cpu"), 1);
    assert_eq!(q.usage("repo"), 0);

    q.call(&["task", "complete", "owner"]);
    let output = chained.output();

    assert!(
        output.status.success(),
        "{}",
        fs::read_to_string(q.log("chained")).unwrap()
    );
    assert_eq!(q.state("chained"), "completed");
    assert_eq!(q.usage("cpu"), 0);
    assert_eq!(q.usage("repo"), 0);

    q.call(&["task", "create", "held", "-tw", "cpu"]);

    assert_eq!(q.state("held"), "running");
    assert_eq!(q.usage("cpu"), 1);
    q.call(&["task", "complete", "held"]);
    assert_eq!(
        q.call(&[
            "task",
            "create",
            "long",
            "--topic",
            "cpu",
            "--wait",
            "--complete"
        ]),
        "successfully registered task 'long'\n'long' started\n'long' successfully completed"
    );
}

#[test]
fn forcing_a_chained_wait_stops_before_start_or_automatic_completion() {
    let q = Queue::new();
    q.topic("cpu");
    q.task("owner", &["cpu"]);
    q.call(&["task", "wait", "owner"]);
    let chained = TestProcess::spawn(
        q.cmd(&["task", "create", "pending", "-twc", "cpu"])
            .stdout(Stdio::piped())
            .stderr(fs::File::create(q.log("pending-chain")).unwrap()),
    );
    q.wait_for_log("pending-chain", "waiting for 'pending'");

    q.call(&["task", "complete", "pending", "-f"]);
    let output = chained.output();

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "successfully registered task 'pending'\n"
    );
    assert!(
        fs::read_to_string(q.log("pending-chain"))
            .unwrap()
            .contains("already completed")
    );
    assert_eq!(q.state("owner"), "running");
    assert_eq!(q.state("pending"), "completed");
    assert_eq!(q.usage("cpu"), 1);
}

#[test]
fn task_list_shortcut_includes_every_state_and_preserves_ownership() {
    let q = Queue::new();

    assert_eq!(q.call(&["task", "--list"]), "");
    q.topic("cpu");
    q.task("running", &["cpu"]);
    q.call(&["task", "wait", "running"]);
    q.task("registered", &[]);
    q.task("queued", &["cpu"]);
    q.task("completed", &[]);
    q.call(&["task", "complete", "-f", "completed"]);

    assert_eq!(
        q.call(&["task", "--list"]),
        "- completed\n- queued\n- registered\n- running"
    );
    assert_eq!(q.call(&["task", "-l"]), q.call(&["task", "--list"]));
    assert_eq!(q.usage("cpu"), 1);
    assert_eq!(q.state("queued"), "queued");
}

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
        q.call(&["task", "create", "my-task"]),
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

    assert_eq!(
        q.call(&[
            "task",
            "link",
            "my-task",
            "itzdesign-commands",
            "itzdesign-repo-lock"
        ]),
        "successfully added 'my-task' to topic 'itzdesign-commands'\nsuccessfully added 'my-task' to topic 'itzdesign-repo-lock'"
    );
    assert_eq!(q.usage("itzdesign-commands"), 1);
    assert_eq!(q.call(&["task", "wait", "my-task"]), "'my-task' started");
    assert_eq!(q.state("my-task"), "running");
    assert_eq!(q.usage("itzdesign-commands"), 1);
    assert_eq!(q.usage("itzdesign-repo-lock"), 1);
    assert!(q.raw(&["task", "wait", "my-task"]).status.success());
    assert!(
        !q.raw(&["topic", "link", "pixel-10-emulator", "my-task"])
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
    assert!(!q.raw(&["task", "create", "my-task"]).status.success());
    assert!(
        !q.raw(&["task", "complete", "--force", "unknown"])
            .status
            .success()
    );
    assert!(
        !q.raw(&["topic", "link", "unknown", "my-task"])
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
fn two_processes_and_repeated_wait_observe_the_same_running_task() {
    let q = Queue::new();
    q.topic("cpu");
    q.task("owner", &["cpu"]);
    q.task("build", &["cpu"]);

    let first = q.spawn_wait("build", "first");
    let second = q.spawn_wait("build", "second");
    q.wait_for_log("first", "waiting for 'build'");
    q.wait_for_log("second", "waiting for 'build'");
    q.call(&["task", "complete", "owner"]);
    let mut codes = [first.output().status.code(), second.output().status.code()];
    codes.sort();

    assert_eq!(codes, [Some(0), Some(0)]);
    q.call(&["task", "wait", "build"]);
    assert_eq!(q.state("build"), "running");
    assert_eq!(q.usage("cpu"), 1);
}

#[test]
fn crossing_topic_links_use_global_task_order_across_processes() {
    let q = Queue::new();
    q.topic("cpu");
    q.topic("repo");
    q.task("owner", &["cpu"]);
    q.task("first", &["cpu"]);
    q.task("second", &["cpu"]);
    q.call(&["topic", "link", "repo", "second", "first"]);
    let mut second = q.spawn_wait("second", "second");
    q.wait_for_log("second", "waiting for 'second'");

    q.call(&["task", "complete", "owner"]);
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
    assert!(!q.raw(&["topic", "capacity", "cpu", "1"]).status.success());

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
            .env("HOME", q.dir.path())
            .env("USERPROFILE", q.dir.path())
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
        q.cmd(&["topic", "link", "repo", "build"])
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
fn topic_command_names_and_leading_dashes_cannot_bypass_domain_validation() {
    let q = Queue::new();

    for name in [
        "create", "capacity", "link", "help", "-cpu", "--list", "-", "--",
    ] {
        let output = q.raw(&["topic", "create", "--", name]);

        assert_eq!(output.status.code(), Some(2), "{name}");
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(
            error.contains("reserved") || error.contains("must start"),
            "{name}: {error}"
        );
        assert!(!q.db.exists(), "{name}");
    }

    q.call(&["topic", "create", "`add`", "--capacity=2"]);
    q.task("`wait`", &["`add`"]);

    q.call(&["topic", "capacity", "`add`", "3"]);
    q.call(&["task", "wait", "`wait`"]);

    assert_eq!(q.state("wait"), "running");
    assert_eq!(q.usage("add"), 1);
}

#[test]
fn help_and_syntax_errors_do_not_open_a_database_and_explicit_path_wins() {
    let q = Queue::new();
    let help = q.raw(&["topic", "create", "--help"]);

    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("topic create [OPTIONS] <TOPIC>"));
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
fn incompatible_database_returns_a_clear_error_without_modifying_it() {
    let q = Queue::new();
    let connection = Connection::open(&q.db).unwrap();
    connection
        .execute_batch(
            "CREATE TABLE unrelated(value TEXT); INSERT INTO unrelated VALUES ('preserve-me'); PRAGMA user_version=4",
        )
        .unwrap();
    drop(connection);
    let before = fs::read(&q.db).unwrap();

    let result = q.raw(&["topic", "--list"]);

    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr).contains("unsupported database schema"));
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
            .env("HOME", q.dir.path())
            .env("USERPROFILE", q.dir.path())
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
            .env("USERPROFILE", q.dir.path())
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
#[path = "support/daemon.rs"]
mod daemon;
