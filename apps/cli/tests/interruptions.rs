#[path = "support/interrupt_process.rs"]
mod interrupt_process;
mod support;

use interrupt_process::InterruptProcess;
use std::fs;
use support::queue::Queue;

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
        ("sqlite.after-commit", 1, Some("queued")),
        ("cli.after-registration", 1, Some("queued")),
        ("sqlite.before-commit", 2, Some("queued")),
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
            queue.task("job", if linked { &["cpu"] } else { &[] });
            InterruptProcess::spawn(&queue, &["task", "job", "complete", "-f"], stage, 1)
                .interrupt();

            let initial = if linked { "queued" } else { "registered" };
            assert_eq!(
                queue.state("job"),
                if committed { "completed" } else { initial }
            );
            assert_eq!(queue.usage("cpu"), 0);
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
        assert_eq!(queue.usage("cpu"), 0);
        assert_eq!(queue.usage("repo"), 0);

        if present {
            assert_eq!(
                queue.call(&["task", "job"]),
                "task: job\nstatus: queued\ntopics:\n- cpu\n- repo"
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
            assert_eq!(queue.usage("repo"), 0);
        }
    }
}

#[test]
fn ctrl_c_admission_boundary_retains_only_committed_ownership_until_manual_complete() {
    for command in [
        vec!["task", "wait", "job"],
        vec!["task", "job", "wait", "-c"],
    ] {
        for (stage, running) in [
            ("sqlite.after-task-update", false),
            ("sqlite.after-membership-write", false),
            ("sqlite.before-commit", false),
            ("sqlite.after-commit", true),
            ("cli.after-admission", true),
            ("cli.before-completion", true),
        ] {
            if stage == "cli.before-completion" && !command.contains(&"-c") {
                continue;
            }

            let queue = Queue::new();
            resources(&queue);
            queue.task("job", &["cpu", "repo"]);
            queue.task("next", &["cpu", "repo"]);
            InterruptProcess::spawn(&queue, &command, stage, 1).interrupt();

            assert_eq!(
                queue.state("job"),
                if running { "running" } else { "queued" },
                "{stage}"
            );
            assert_eq!(queue.usage("cpu"), i64::from(running));
            assert_eq!(queue.usage("repo"), i64::from(running));

            let mut waiting = queue.spawn_wait("next", "next");
            queue.wait_for_log("next", "waiting for 'next'");
            waiting.assert_running();
            waiting.stop();

            queue.call(&["task", "complete", "job", "-f"]);
            queue.call(&["task", "wait", "next"]);
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
    assert_eq!(queue.state("job"), "queued");
    queue.call(&["task", "job", "wait"]);
    assert_eq!(queue.usage("cpu"), 1);
    assert_eq!(queue.usage("repo"), 1);
}

#[test]
fn ctrl_c_completion_boundary_keeps_all_slots_or_releases_all_slots() {
    for (stage, completed) in [
        ("cli.before-completion", false),
        ("sqlite.after-task-update", false),
        ("sqlite.after-membership-write", false),
        ("sqlite.before-commit", false),
        ("sqlite.after-commit", true),
        ("cli.after-completion", true),
    ] {
        let queue = Queue::new();
        resources(&queue);
        owned(&queue, "job");
        InterruptProcess::spawn(&queue, &["task", "job", "complete"], stage, 1).interrupt();

        assert_eq!(
            queue.state("job"),
            if completed { "completed" } else { "running" },
            "{stage}"
        );
        assert_eq!(queue.usage("cpu"), i64::from(!completed));
        assert_eq!(queue.usage("repo"), i64::from(!completed));

        if !completed {
            queue.call(&["task", "complete", "job"]);
        }

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
            assert_eq!(queue.state("job"), "queued");
            assert_eq!(queue.usage("cpu"), 0);
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
    assert_eq!(queue.state("waiting"), "queued");
}
