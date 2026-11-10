#[path = "support/interrupt_process.rs"]
mod interrupt_process;
#[path = "support/native_interrupt.rs"]
mod native_interrupt;
mod support;

use interrupt_process::InterruptProcess;
use std::fs;
use support::queue::Queue;

#[test]
fn native_owner_ctrl_c_rolls_back_or_preserves_its_active_client_commit() {
    for (stage, committed) in [
        ("sqlite.before-commit", false),
        ("sqlite.after-commit", true),
    ] {
        let queue = Queue::new();
        let owner = queue.spawn_owner();
        resources(&queue);
        let client = InterruptProcess::spawn(
            &queue,
            &["task", "create", "job", "-t", "cpu", "-t", "repo"],
            stage,
            1,
        );

        owner.interrupt();
        assert!(!client.output().status.success());

        assert_eq!(queue.call(&["daemon", "status"]), "daemon is stopped");
        assert_eq!(
            queue.call(&["task", "list"]),
            if committed { "- job" } else { "" }
        );
        assert_eq!(queue.usage("cpu"), i64::from(committed));
        assert_eq!(queue.usage("repo"), i64::from(committed));
        if committed {
            queue.call(&["task", "complete", "job"]);
        }

        queue.task("next", &["cpu", "repo"]);
        assert_eq!(queue.state("next"), "running");
    }
}

#[test]
fn lost_owner_response_reports_uncertainty_without_replaying_committed_registration() {
    let queue = Queue::new();
    let owner = queue.spawn_owner();
    resources(&queue);
    let client = InterruptProcess::spawn(
        &queue,
        &["task", "create", "job", "-t", "cpu", "-t", "repo"],
        "sqlite.after-commit",
        1,
    );

    owner.stop();
    let result = client.output();

    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr).contains("outcome is uncertain"));
    assert_eq!(queue.call(&["daemon", "status"]), "daemon is stopped");
    assert_eq!(queue.state("job"), "running");
    assert_eq!(queue.usage("cpu"), 1);
    assert_eq!(queue.usage("repo"), 1);
    assert_eq!(queue.call(&["task", "list"]), "- job");
}

#[test]
fn ctrl_c_interrupts_a_full_stdout_pipe_without_reading_it_or_releasing_ownership() {
    let queue = Queue::new();
    let expected_size = support::large_output::seed_tasks(&queue).len();
    resources(&queue);
    owned(&queue, "owner");
    let mut child = InterruptProcess::spawn(&queue, &["task", "list"], "cli.before-operation", 1);

    child.resume();
    assert_eq!(child.read_stdout_byte(), b'-');
    let output = child.interrupt();

    assert!(output.stdout.len() + 1 < expected_size);
    assert_eq!(queue.state("owner"), "running");
    assert_eq!(queue.usage("cpu"), 1);
    assert_eq!(queue.usage("repo"), 1);

    queue.call(&["task", "complete", "owner"]);
    queue.call(&["task", "create", "next", "-twc", "cpu", "-t", "repo"]);

    assert_eq!(queue.state("next"), "completed");
    assert_eq!(queue.usage("cpu"), 0);
    assert_eq!(queue.usage("repo"), 0);
}

#[test]
fn ctrl_c_interrupts_a_full_stderr_pipe_without_changing_config_or_ownership() {
    let queue = Queue::new();
    resources(&queue);
    owned(&queue, "owner");
    let path = queue.dir.path().join(".anyq/config.toml");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let contents = format!("{} = true\n", "x".repeat(2 * 1024 * 1024));
    fs::write(&path, &contents).unwrap();
    let mut child = InterruptProcess::spawn(&queue, &["config", "list"], "cli.before-command", 1);

    child.resume();
    assert_eq!(child.read_stderr_byte(), b'e');
    let output = child.interrupt();

    assert!(output.stderr.len() + 1 < contents.len());
    assert_eq!(fs::read_to_string(&path).unwrap(), contents);
    assert_eq!(queue.state("owner"), "running");
    assert_eq!(queue.usage("cpu"), 1);
    assert_eq!(queue.usage("repo"), 1);

    fs::write(&path, "").unwrap();
    queue.call(&["config", "task.autocreate", "true"]);
    queue.call(&["task", "complete", "owner"]);

    assert_eq!(queue.usage("cpu"), 0);
    assert_eq!(queue.usage("repo"), 0);
}

fn resources(queue: &Queue) {
    queue.topic("cpu");
    queue.topic("repo");
}

fn owned(queue: &Queue, name: &str) {
    queue.task(name, &["cpu", "repo"]);
    queue.call(&["task", "wait", name]);
}

#[test]
fn ctrl_c_at_each_create_wait_complete_commit_keeps_exactly_the_committed_phase() {
    for (stage, hit, expected) in [
        ("sqlite.before-commit", 1, None),
        ("sqlite.after-commit", 1, Some("running")),
        ("cli.after-registration", 1, Some("running")),
        ("sqlite.before-commit", 2, Some("running")),
        ("sqlite.after-commit", 2, Some("running")),
        ("cli.after-admission", 1, Some("running")),
        ("cli.before-completion", 1, Some("running")),
        ("sqlite.before-commit", 3, Some("running")),
        ("sqlite.after-commit", 3, Some("completed")),
        ("cli.after-completion", 1, Some("completed")),
    ] {
        let queue = Queue::new();
        resources(&queue);
        InterruptProcess::spawn(
            &queue,
            &["task", "job", "create", "-twc", "cpu", "-t", "repo"],
            stage,
            hit,
        )
        .interrupt();

        assert_eq!(
            queue.call(&["task", "list"]),
            if expected.is_some() { "- job" } else { "" },
            "{stage} #{hit}"
        );

        if let Some(state) = expected {
            assert_eq!(queue.state("job"), state, "{stage} #{hit}");
        }

        assert_eq!(queue.usage("cpu"), i64::from(expected == Some("running")));
        assert_eq!(queue.usage("repo"), i64::from(expected == Some("running")));

        if expected.is_some() && expected != Some("completed") {
            queue.call(&["task", "complete", "job", "-f"]);
        }

        queue.call(&["task", "next", "create", "-twc", "cpu", "-t", "repo"]);
        assert_eq!(queue.state("next"), "completed");
    }
}

#[test]
fn ctrl_c_topic_create_and_capacity_changes_keep_old_or_whole_new_values() {
    for action in ["create", "capacity"] {
        for (stage, committed) in [
            ("sqlite.after-topic-write", false),
            ("sqlite.before-commit", false),
            ("sqlite.after-commit", true),
            ("cli.after-command", true),
        ] {
            let queue = Queue::new();

            if action == "capacity" {
                queue.topic("cpu");
                owned_on_cpu(&queue);
            }

            let args = if action == "create" {
                vec!["topic", "cpu", "create", "-c", "2"]
            } else {
                vec!["topic", "cpu", "capacity", "2"]
            };
            InterruptProcess::spawn(&queue, &args, stage, 1).interrupt();

            if action == "create" && !committed {
                assert_eq!(queue.call(&["topic", "list"]), "");
            } else {
                assert_eq!(
                    queue.call(&["topic", "cpu", "capacity"]),
                    if committed { "2" } else { "1" }
                );
            }

            if action == "capacity" {
                assert_eq!(queue.state("owner"), "running");
                assert_eq!(queue.usage("cpu"), 1);
                queue.call(&["task", "complete", "owner"]);
            }
        }
    }
}

fn owned_on_cpu(queue: &Queue) {
    queue.task("owner", &["cpu"]);
    queue.call(&["task", "wait", "owner"]);
}

#[test]
fn ctrl_c_force_completion_of_unstarted_tasks_preserves_the_last_committed_state() {
    for linked in [false, true] {
        for (stage, committed) in [
            ("sqlite.after-task-update", false),
            ("sqlite.before-commit", false),
            ("sqlite.after-commit", true),
            ("cli.after-completion", true),
        ] {
            let queue = Queue::new();
            queue.topic("cpu");
            if linked {
                owned_on_cpu(&queue);
            }
            queue.task("job", if linked { &["cpu"] } else { &[] });
            InterruptProcess::spawn(&queue, &["task", "job", "complete", "-f"], stage, 1)
                .interrupt();

            let initial = if linked { "queued" } else { "registered" };
            assert_eq!(
                queue.state("job"),
                if committed { "completed" } else { initial }
            );
            assert_eq!(queue.usage("cpu"), i64::from(linked));
        }
    }
}

#[test]
fn ctrl_c_autocreating_read_rolls_back_before_commit_and_never_grants_admission() {
    for kind in ["task", "topic"] {
        for (stage, committed) in [
            ("sqlite.before-commit", false),
            ("sqlite.after-commit", true),
            ("cli.after-command", true),
        ] {
            let queue = Queue::new();
            queue.call(&["config", "task.autocreate", "true"]);
            queue.call(&["config", "topic.autocreate", "true"]);
            InterruptProcess::spawn(&queue, &[kind, "new"], stage, 1).interrupt();

            assert_eq!(
                queue.call(&[kind, "list"]),
                if committed { "- new" } else { "" }
            );

            if committed && kind == "task" {
                assert_eq!(queue.state("new"), "registered");
            }
        }
    }

    for command in [
        vec!["task", "wait", "missing"],
        vec!["task", "complete", "missing"],
    ] {
        let queue = Queue::new();
        queue.call(&["config", "task.autocreate", "true"]);
        InterruptProcess::spawn(&queue, &command, "sqlite.after-task-insert", 1).interrupt();

        assert_eq!(queue.call(&["task", "list"]), "");
    }
}

#[test]
fn ctrl_c_config_reads_unset_and_path_writes_obey_the_same_atomic_boundary() {
    for command in [
        vec!["config", "list"],
        vec!["config", "--list"],
        vec!["config", "-l"],
        vec!["config", "task.autocreate"],
        vec!["config", "--get", "db.path"],
    ] {
        for stage in [
            "config.before-read",
            "config.after-read",
            "cli.after-command",
        ] {
            let queue = Queue::new();
            queue.call(&["config", "db.path", "old.db"]);
            let path = queue.dir.path().join(".anyq/config.toml");
            let before = fs::read(&path).unwrap();
            InterruptProcess::spawn(&queue, &command, stage, 1).interrupt();

            assert_eq!(fs::read(path).unwrap(), before);
            assert!(!queue.db.exists());
        }
    }

    for command in [
        vec!["config", "--unset", "db.path"],
        vec!["config", "db.path", "new directory/new.db"],
    ] {
        for (stage, replaced) in [
            ("config.before-replace", false),
            ("config.after-replace", true),
        ] {
            let queue = Queue::new();
            queue.call(&["config", "db.path", "old.db"]);
            InterruptProcess::spawn(&queue, &command, stage, 1).interrupt();

            let output = queue.raw(&["config", "db.path"]);

            if replaced && command.contains(&"--unset") {
                assert_eq!(output.status.code(), Some(1));
            } else {
                assert!(output.status.success());
                assert_eq!(
                    String::from_utf8(output.stdout).unwrap().trim(),
                    if replaced {
                        "new directory/new.db"
                    } else {
                        "old.db"
                    }
                );
            }

            assert!(!queue.db.exists());
            queue.call(&["config", "task.autocreate", "true"]);
        }
    }
}

#[test]
fn ctrl_c_before_parsing_covers_every_command_family_without_creating_data() {
    let commands = [
        vec!["topic", "list"],
        vec!["topic", "-l"],
        vec!["topic", "--list"],
        vec!["topic", "create", "cpu"],
        vec!["topic", "cpu", "create"],
        vec!["topic", "cpu"],
        vec!["topic", "capacity", "cpu"],
        vec!["topic", "cpu", "capacity", "2"],
        vec!["topic", "link", "cpu", "job"],
        vec!["topic", "cpu", "link", "job", "-wc"],
        vec!["task", "list"],
        vec!["task", "-l"],
        vec!["task", "--list"],
        vec!["task", "create", "job"],
        vec!["task", "job", "create", "-twc", "cpu"],
        vec!["task", "job"],
        vec!["task", "link", "job", "cpu", "-wc"],
        vec!["task", "job", "link", "cpu"],
        vec!["task", "wait", "job", "-c"],
        vec!["task", "job", "wait"],
        vec!["task", "complete", "job"],
        vec!["task", "job", "complete", "-f"],
        vec!["config", "list"],
        vec!["config", "--list"],
        vec!["config", "task.autocreate", "true"],
        vec!["config", "--get", "db.path"],
        vec!["config", "--unset", "db.path"],
        vec!["help"],
        vec!["help", "task", "link"],
        vec!["-h"],
        vec!["-v"],
    ];

    for command in commands {
        let queue = Queue::new();
        InterruptProcess::spawn(&queue, &command, "cli.before-parse", 1).interrupt();

        assert!(!queue.db.exists(), "{command:?}");
        assert!(
            !queue.dir.path().join(".anyq/config.toml").exists(),
            "{command:?}"
        );
    }
}

#[test]
fn ctrl_c_at_database_initialization_boundaries_leaves_a_reopenable_database() {
    for stage in [
        "cli.before-command",
        "config.before-read",
        "config.after-read",
        "sqlite.before-open",
        "sqlite.after-open",
        "sqlite.before-wal",
        "sqlite.schema.before-commit",
        "sqlite.schema.after-commit",
    ] {
        let queue = Queue::new();
        InterruptProcess::spawn(&queue, &["task", "list"], stage, 1).interrupt();

        assert_eq!(queue.call(&["task", "list"]), "", "{stage}");
        assert_eq!(queue.call(&["topic", "list"]), "", "{stage}");
    }
}

#[test]
fn ctrl_c_during_registration_rolls_back_partial_memberships_until_commit() {
    for (stage, present) in [
        ("sqlite.after-task-insert", false),
        ("sqlite.after-task-update", false),
        ("sqlite.after-membership-write", false),
        ("sqlite.before-commit", false),
        ("sqlite.after-commit", true),
        ("cli.after-registration", true),
    ] {
        let queue = Queue::new();
        resources(&queue);
        InterruptProcess::spawn(
            &queue,
            &["task", "create", "job", "-t", "cpu", "-t", "repo"],
            stage,
            1,
        )
        .interrupt();

        assert_eq!(
            queue.call(&["task", "list"]),
            if present { "- job" } else { "" },
            "{stage}"
        );
        assert_eq!(queue.usage("cpu"), i64::from(present));
        assert_eq!(queue.usage("repo"), i64::from(present));

        if present {
            assert_eq!(
                queue.call(&["task", "job"]),
                "task: job\nstatus: running\ntopics:\n- cpu\n- repo"
            );
            queue.call(&["task", "complete", "job", "-f"]);
        } else {
            queue.task("job", &["cpu", "repo"]);
        }
    }
}

#[test]
fn ctrl_c_during_link_preserves_the_previous_or_the_whole_new_membership() {
    for command in [
        vec!["topic", "link", "cpu", "job", "-wc"],
        vec!["topic", "cpu", "link", "job", "-wc"],
        vec!["task", "link", "job", "cpu", "-wc"],
        vec!["task", "job", "link", "cpu", "-wc"],
    ] {
        for (stage, linked) in [
            ("sqlite.after-task-update", false),
            ("sqlite.after-membership-write", false),
            ("sqlite.before-commit", false),
            ("sqlite.after-commit", true),
            ("cli.after-link", true),
        ] {
            let queue = Queue::new();
            resources(&queue);
            queue.task("owner", &["repo"]);
            queue.task("job", &["repo"]);
            InterruptProcess::spawn(&queue, &command, stage, 1).interrupt();

            let expected = if linked {
                "task: job\nstatus: queued\ntopics:\n- cpu\n- repo"
            } else {
                "task: job\nstatus: queued\ntopics:\n- repo"
            };
            assert_eq!(
                queue.call(&["task", "job"]),
                expected,
                "{command:?} {stage}"
            );
            assert_eq!(queue.usage("cpu"), 0);
            assert_eq!(queue.usage("repo"), 1);
        }
    }
}

#[test]
fn ctrl_c_admission_boundary_retains_only_committed_ownership_until_manual_complete() {
    for command in [
        vec!["task", "wait", "job"],
        vec!["task", "job", "wait", "-c"],
    ] {
        for stage in [
            "sqlite.before-commit",
            "sqlite.after-commit",
            "cli.after-admission",
            "cli.before-completion",
        ] {
            if stage == "cli.before-completion" && !command.contains(&"-c") {
                continue;
            }

            let queue = Queue::new();
            resources(&queue);
            queue.task("job", &["cpu", "repo"]);
            queue.task("next", &["cpu", "repo"]);

            InterruptProcess::spawn(&queue, &command, stage, 1).interrupt();

            assert_eq!(queue.state("job"), "running", "{stage}");
            assert_eq!(queue.state("next"), "queued");
            assert_eq!(queue.usage("cpu"), 1);
            assert_eq!(queue.usage("repo"), 1);

            let mut waiting = queue.spawn_wait("next", "next");
            queue.wait_for_log("next", "waiting for 'next'");
            waiting.assert_running();
            waiting.stop();

            queue.call(&["task", "complete", "job"]);

            assert_eq!(queue.state("next"), "running");
            queue.call(&["task", "complete", "next"]);
        }
    }
}

#[test]
fn ctrl_c_while_waiting_never_reserves_the_other_free_resource() {
    let queue = Queue::new();
    resources(&queue);
    queue.task("owner", &["repo"]);
    queue.call(&["task", "wait", "owner"]);
    queue.task("job", &["cpu", "repo"]);

    InterruptProcess::spawn(&queue, &["task", "job", "wait", "-c"], "cli.waiting", 1).interrupt();

    assert_eq!(queue.state("job"), "queued");
    assert_eq!(queue.usage("cpu"), 0);
    assert_eq!(queue.usage("repo"), 1);

    queue.call(&["task", "complete", "owner"]);
    assert_eq!(queue.state("job"), "running");
    queue.call(&["task", "job", "wait"]);
    assert_eq!(queue.usage("cpu"), 1);
    assert_eq!(queue.usage("repo"), 1);
}

#[test]
fn ctrl_c_completion_boundary_keeps_all_slots_or_releases_all_slots() {
    for (stage, hit, completed) in [
        ("cli.before-completion", 1, false),
        ("sqlite.after-task-update", 1, false),
        ("sqlite.after-task-update", 2, false),
        ("sqlite.after-membership-write", 1, false),
        ("sqlite.after-membership-write", 3, false),
        ("sqlite.before-commit", 1, false),
        ("sqlite.after-commit", 1, true),
        ("cli.after-completion", 1, true),
    ] {
        let queue = Queue::new();
        resources(&queue);
        owned(&queue, "job");
        queue.task("next", &["cpu", "repo"]);

        InterruptProcess::spawn(&queue, &["task", "job", "complete"], stage, hit).interrupt();

        assert_eq!(
            queue.state("job"),
            if completed { "completed" } else { "running" },
            "{stage} #{hit}"
        );
        assert_eq!(
            queue.state("next"),
            if completed { "running" } else { "queued" }
        );
        assert_eq!(queue.usage("cpu"), 1);
        assert_eq!(queue.usage("repo"), 1);

        if !completed {
            queue.call(&["task", "complete", "job"]);
        }

        assert_eq!(queue.state("next"), "running");
        queue.call(&["task", "complete", "next"]);
        assert_eq!(queue.usage("cpu"), 0);
        assert_eq!(queue.usage("repo"), 0);
    }
}

#[test]
fn ctrl_c_during_autocreation_rolls_back_both_new_objects_until_link_commit() {
    for (stage, present) in [
        ("sqlite.after-task-insert", false),
        ("sqlite.after-topic-write", false),
        ("sqlite.after-membership-write", false),
        ("sqlite.before-commit", false),
        ("sqlite.after-commit", true),
    ] {
        let queue = Queue::new();
        queue.call(&["config", "task.autocreate", "true"]);
        queue.call(&["config", "topic.autocreate", "true"]);
        InterruptProcess::spawn(&queue, &["task", "job", "link", "cpu"], stage, 1).interrupt();

        assert_eq!(
            queue.call(&["task", "list"]),
            if present { "- job" } else { "" },
            "{stage}"
        );
        assert_eq!(
            queue.call(&["topic", "list"]),
            if present { "- cpu" } else { "" },
            "{stage}"
        );

        if present {
            assert_eq!(queue.state("job"), "running");
            assert_eq!(queue.usage("cpu"), 1);
        }
    }
}

#[test]
fn ctrl_c_read_commands_do_not_release_existing_ownership() {
    for command in [
        vec!["task", "job"],
        vec!["task", "list"],
        vec!["task", "-l"],
        vec!["topic", "cpu"],
        vec!["topic", "list"],
        vec!["topic", "--list"],
        vec!["topic", "capacity", "cpu"],
        vec!["topic", "cpu", "capacity"],
    ] {
        for stage in [
            "cli.before-operation",
            "sqlite.before-commit",
            "cli.after-command",
        ] {
            let queue = Queue::new();
            resources(&queue);
            owned(&queue, "job");
            InterruptProcess::spawn(&queue, &command, stage, 1).interrupt();

            assert_eq!(queue.state("job"), "running", "{command:?} {stage}");
            assert_eq!(queue.usage("cpu"), 1);
            assert_eq!(queue.usage("repo"), 1);
        }
    }
}

#[test]
fn ctrl_c_configuration_replace_is_atomic_and_cleans_temporary_files_and_lock() {
    for (stage, replaced) in [
        ("config.before-change", false),
        ("config.before-lock", false),
        ("config.after-read", false),
        ("config.before-replace", false),
        ("config.after-replace", true),
    ] {
        let queue = Queue::new();
        queue.call(&["config", "task.autocreate", "true"]);
        let config = queue.dir.path().join(".anyq/config.toml");
        let before = fs::read(&config).unwrap();

        InterruptProcess::spawn(&queue, &["config", "task.autocreate", "false"], stage, 1)
            .interrupt();

        if !replaced {
            assert_eq!(fs::read(&config).unwrap(), before, "{stage}");
        }

        assert_eq!(
            queue.call(&["config", "task.autocreate"]),
            if replaced { "false" } else { "true" }
        );
        assert!(!queue.db.exists());
        assert_eq!(fs::read_dir(config.parent().unwrap()).unwrap().count(), 2);
        queue.call(&["config", "topic.autocreate", "true"]);
        assert_eq!(queue.call(&["config", "topic.autocreate"]), "true");
    }
}

#[test]
fn ctrl_c_while_waiting_for_config_lock_does_not_steal_or_leave_the_lock() {
    let queue = Queue::new();
    queue.call(&["config", "task.autocreate", "false"]);
    let holder = InterruptProcess::spawn(
        &queue,
        &["config", "task.autocreate", "true"],
        "config.before-replace",
        1,
    );
    let waiting = InterruptProcess::spawn(
        &queue,
        &["config", "topic.autocreate", "true"],
        "config.waiting-lock",
        1,
    );

    waiting.interrupt();
    assert!(holder.release().status.success());

    assert_eq!(queue.call(&["config", "task.autocreate"]), "true");
    assert_eq!(queue.call(&["config", "topic.autocreate"]), "false");
    queue.call(&["config", "topic.autocreate", "true"]);
    assert_eq!(queue.call(&["config", "topic.autocreate"]), "true");
}

#[test]
fn ctrl_c_while_waiting_for_sqlite_lock_leaves_the_owner_transaction_intact() {
    let queue = Queue::new();
    resources(&queue);
    let holder = InterruptProcess::spawn(
        &queue,
        &["task", "create", "owner"],
        "sqlite.before-commit",
        1,
    );
    let waiting = InterruptProcess::spawn(
        &queue,
        &["task", "create", "waiting"],
        "sqlite.waiting-lock",
        1,
    );

    waiting.interrupt();
    assert!(holder.release().status.success());

    assert_eq!(queue.call(&["task", "list"]), "- owner");
    assert_eq!(queue.state("owner"), "registered");
    queue.task("waiting", &["cpu"]);
    assert_eq!(queue.state("waiting"), "running");
}

#[test]
fn ctrl_c_batch_links_never_keep_partial_tasks_or_resources() {
    for task_batch in [false, true] {
        for (stage, hit, committed) in [
            ("sqlite.after-task-update", 1, false),
            ("sqlite.after-task-update", 2, false),
            ("sqlite.after-membership-write", 2, false),
            ("sqlite.before-commit", 1, false),
            ("sqlite.after-commit", 1, true),
            ("cli.after-link", 1, true),
        ] {
            let queue = Queue::new();
            resources(&queue);
            queue.task("first", &[]);
            queue.task("second", &[]);
            let args = if task_batch {
                vec!["topic", "link", "cpu", "second", "first"]
            } else {
                vec!["task", "link", "first", "cpu", "repo"]
            };

            InterruptProcess::spawn(&queue, &args, stage, hit).interrupt();

            assert_eq!(
                queue.state("first"),
                if committed { "running" } else { "registered" }
            );
            assert_eq!(
                queue.state("second"),
                if committed && task_batch {
                    "queued"
                } else {
                    "registered"
                }
            );
            assert_eq!(queue.usage("cpu"), i64::from(committed));
            assert_eq!(queue.usage("repo"), i64::from(committed && !task_batch));
            let output = queue.call(&["task", "first"]);
            assert_eq!(output.contains("- cpu"), committed);
            assert_eq!(output.contains("- repo"), committed && !task_batch);
        }
    }
}

#[test]
fn ctrl_c_capacity_change_rolls_back_all_automatic_promotions_before_commit() {
    for (stage, hit, committed) in [
        ("sqlite.after-topic-write", 1, false),
        ("sqlite.after-task-update", 1, false),
        ("sqlite.after-task-update", 2, false),
        ("sqlite.before-commit", 1, false),
        ("sqlite.after-commit", 1, true),
    ] {
        let queue = Queue::new();
        queue.topic("cpu");
        for name in ["owner", "second", "third"] {
            queue.task(name, &["cpu"]);
        }

        InterruptProcess::spawn(&queue, &["topic", "capacity", "cpu", "3"], stage, hit).interrupt();

        assert_eq!(
            queue.call(&["topic", "capacity", "cpu"]),
            if committed { "3" } else { "1" }
        );
        for name in ["second", "third"] {
            assert_eq!(
                queue.state(name),
                if committed { "running" } else { "queued" }
            );
        }
        assert_eq!(queue.usage("cpu"), if committed { 3 } else { 1 });
    }
}

#[test]
fn ctrl_c_explicit_start_keeps_the_stopped_task_or_commits_all_slots() {
    for (stage, committed) in [
        ("sqlite.after-task-update", false),
        ("sqlite.after-membership-write", false),
        ("sqlite.before-commit", false),
        ("sqlite.after-commit", true),
        ("cli.after-admission", true),
    ] {
        let queue = Queue::new();
        resources(&queue);
        queue.call(&["task", "create", "job", "-t", "cpu", "-t", "repo", "--stop"]);

        InterruptProcess::spawn(&queue, &["task", "job", "start", "-wc"], stage, 1).interrupt();

        assert_eq!(
            queue.state("job"),
            if committed { "running" } else { "queued" }
        );
        assert_eq!(queue.usage("cpu"), i64::from(committed));
        assert_eq!(queue.usage("repo"), i64::from(committed));
        if !committed {
            queue.call(&["topic", "cpu", "autostart"]);
            assert_eq!(queue.state("job"), "queued");
        }
    }
}

#[test]
fn ctrl_c_start_wait_does_not_enable_a_stopped_task_or_release_the_owner() {
    let queue = Queue::new();
    resources(&queue);
    owned(&queue, "owner");
    queue.call(&["task", "create", "job", "-t", "cpu", "-t", "repo", "--stop"]);

    InterruptProcess::spawn(&queue, &["task", "job", "start", "-wc"], "cli.waiting", 1).interrupt();

    assert_eq!(queue.state("job"), "queued");
    assert_eq!(queue.state("owner"), "running");
    queue.call(&["task", "complete", "owner"]);
    assert_eq!(queue.state("job"), "queued");
    assert_eq!(queue.usage("cpu"), 0);
    assert_eq!(queue.usage("repo"), 0);
}

#[test]
fn ctrl_c_topic_autostart_and_its_promotions_share_one_commit() {
    for (stage, committed) in [
        ("sqlite.after-topic-write", false),
        ("sqlite.after-task-update", false),
        ("sqlite.before-commit", false),
        ("sqlite.after-commit", true),
    ] {
        let queue = Queue::new();
        queue.topic("cpu");
        queue.call(&["topic", "cpu", "autostart", "off"]);
        queue.task("job", &["cpu"]);

        InterruptProcess::spawn(&queue, &["topic", "cpu", "autostart"], stage, 1).interrupt();

        assert_eq!(
            queue.state("job"),
            if committed { "running" } else { "queued" }
        );
        assert_eq!(queue.usage("cpu"), i64::from(committed));
        let connection = rusqlite::Connection::open(&queue.db).unwrap();
        let enabled: bool = connection
            .query_row("SELECT autostart FROM topics WHERE name='cpu'", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(enabled, committed);
    }
}

#[test]
fn ctrl_c_schema_upgrade_preserves_the_old_or_whole_new_schema_and_ownership() {
    for (stage, committed) in [
        ("sqlite.schema.before-commit", false),
        ("sqlite.schema.after-commit", true),
    ] {
        let queue = Queue::new();
        let connection = rusqlite::Connection::open(&queue.db).unwrap();
        connection
            .execute_batch(include_str!(
                "../../../features/queue/impl/sqlite/src/tests/schema_v2.sql"
            ))
            .unwrap();
        drop(connection);

        InterruptProcess::spawn(&queue, &["task", "list"], stage, 1).interrupt();

        let connection = rusqlite::Connection::open(&queue.db).unwrap();
        let version: i64 = connection
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, if committed { 3 } else { 2 });
        assert_eq!(queue.state("owner"), "running");
        assert_eq!(queue.state("next"), "queued");
        assert_eq!(queue.usage("cpu"), 1);
        drop(connection);

        queue.call(&["task", "complete", "owner"]);

        assert_eq!(queue.state("next"), "running");
        assert_eq!(queue.usage("repo"), 1);
    }
}
