use crate::support::{process::TestProcess, queue::Queue};
use std::{fs, process::Stdio};

#[test]
fn list_aliases_name_first_actions_and_nested_help_share_the_same_contract() {
    let q = Queue::new();
    q.call(&["topic", "cpu", "create", "-c", "2"]);
    q.call(&["topic", "repo", "create"]);
    q.call(&["task", "job", "create", "-t", "cpu"]);
    q.call(&["topic", "repo", "link", "job"]);
    q.call(&["task", "job", "link", "repo"]);

    for kind in ["task", "topic"] {
        let expected = q.call(&[kind, "list"]);

        for alias in ["-l", "--list"] {
            assert_eq!(q.call(&[kind, alias]), expected);
        }
    }

    assert_eq!(q.call(&["topic", "cpu", "capacity"]), "2");
    q.call(&["topic", "cpu", "capacity", "3"]);
    assert_eq!(q.call(&["topic", "capacity", "cpu"]), "3");
    q.call(&["task", "job", "wait"]);
    assert_eq!(q.state("job"), "running");
    q.call(&["task", "job", "complete"]);
    assert_eq!(q.state("job"), "completed");

    let clean = Queue::new();
    assert_eq!(clean.call(&["help"]), clean.call(&["--help"]));

    for path in [
        vec!["task"],
        vec!["topic"],
        vec!["config"],
        vec!["config", "list"],
        vec!["task", "create"],
        vec!["task", "link"],
        vec!["task", "wait"],
        vec!["task", "complete"],
        vec!["task", "list"],
        vec!["topic", "create"],
        vec!["topic", "link"],
        vec!["topic", "capacity"],
        vec!["topic", "list"],
    ] {
        let mut help = vec!["help"];
        help.extend(&path);
        let mut direct = path;
        direct.push("--help");

        assert_eq!(clean.call(&help), clean.call(&direct));
    }

    assert_eq!(clean.raw(&["help", "unknown"]).status.code(), Some(2));
    assert!(!clean.db.exists());
    assert!(!clean.dir.path().join(".anyq").exists());
}

#[test]
fn link_and_wait_chains_complete_or_preserve_manual_ownership_in_all_forms() {
    for command in [
        vec!["topic", "link", "cpu", "job", "--wait", "--complete"],
        vec!["topic", "cpu", "link", "job", "-wc"],
        vec!["task", "link", "job", "cpu", "-wc"],
        vec!["task", "job", "link", "cpu", "--wait", "--complete"],
    ] {
        let q = Queue::new();
        q.topic("cpu");
        q.task("job", &[]);

        let output = q.call(&command);

        assert!(output.contains("'job' started"));
        assert!(output.ends_with("'job' successfully completed"));
        assert_eq!(q.state("job"), "completed");
        assert_eq!(q.usage("cpu"), 0);
    }

    for command in [
        vec!["task", "wait", "job", "--complete"],
        vec!["task", "job", "wait", "-c"],
        vec!["topic", "cpu", "link", "job", "-w"],
        vec!["task", "job", "link", "cpu", "--wait"],
    ] {
        let q = Queue::new();
        q.topic("cpu");
        q.task("job", &["cpu"]);
        q.call(&command);

        let completed = command.contains(&"-c") || command.contains(&"--complete");
        assert_eq!(
            q.state("job"),
            if completed { "completed" } else { "running" }
        );
        assert_eq!(q.usage("cpu"), i64::from(!completed));
    }
}

#[test]
fn invalid_chain_dependencies_and_old_add_are_errors_before_database_access() {
    let q = Queue::new();

    for command in [
        vec!["task", "add", "job"],
        vec!["task", "job", "add"],
        vec!["task", "job", "create", "-w"],
        vec!["task", "job", "create", "-tc", "cpu"],
        vec!["task", "link", "job", "cpu", "-c"],
        vec!["task", "job", "link", "cpu", "--complete"],
        vec!["topic", "link", "cpu", "job", "--complete"],
        vec!["topic", "cpu", "link", "job", "-c"],
    ] {
        assert_eq!(q.raw(&command).status.code(), Some(2), "{command:?}");
        assert!(!q.db.exists(), "{command:?}");
    }
}

#[test]
fn quoted_keywords_empty_names_and_shell_escaped_spaces_round_trip() {
    let q = Queue::new();

    for word in [
        "list", "create", "capacity", "link", "add", "wait", "complete", "config", "help", "",
        "-x", "--x",
    ] {
        let quoted = format!("`{word}`");
        q.call(&["topic", &quoted, "create"]);
        q.call(&["task", &quoted, "create", "-twc", &quoted]);

        assert_eq!(q.state(word), "completed");
        assert!(q.call(&["task", &quoted]).contains("status: completed"));
        assert_eq!(q.call(&["topic", &quoted, "capacity"]), "1");
    }

    let spelling = "build with spaces";
    q.call(&["task", spelling, "create", "-twc", "`list`"]);

    assert_eq!(q.state(spelling), "completed");
}

#[cfg(unix)]
#[test]
fn shell_escaped_spaces_arrive_as_one_valid_name() {
    let q = Queue::new();
    let output = TestProcess::spawn(
        std::process::Command::new("/bin/sh")
            .args([
                "-c",
                "\"$ANYQ_TEST_BINARY\" task create build\\ with\\ spaces",
            ])
            .env("ANYQ_TEST_BINARY", env!("CARGO_BIN_EXE_anyq"))
            .env("ANYQ_DB", &q.db)
            .env("HOME", q.dir.path())
            .env("USERPROFILE", q.dir.path())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped()),
    )
    .output();

    assert!(output.status.success());
    assert_eq!(q.state("build with spaces"), "registered");
}

#[test]
fn literal_newlines_backslashes_and_nested_backticks_survive_the_full_lifecycle() {
    let q = Queue::new();

    for (index, name) in ["`line\nnext`", "`\\ escaped \\ `", "```nested```", "a\\ "]
        .into_iter()
        .enumerate()
    {
        let topic = format!("cpu{index}");
        q.topic(&topic);
        q.call(&["task", name, "create", "-twc", &topic]);

        assert!(q.call(&["task", name]).contains("status: completed"));
        assert_eq!(q.usage(&topic), 0);
    }
}

#[test]
fn configuration_defaults_get_set_unset_and_errors_never_open_the_queue() {
    let q = Queue::new();
    let path = q.dir.path().join(".anyq/config.toml");

    assert_eq!(
        q.call(&["config", "-l"]),
        "task.autocreate=false\ntopic.autocreate=false"
    );
    assert_eq!(q.call(&["config", "--list"]), q.call(&["config", "-l"]));
    assert_eq!(q.call(&["config", "list"]), q.call(&["config", "-l"]));
    assert_eq!(q.call(&["config", "--get", "task.autocreate"]), "false");
    assert_eq!(q.raw(&["config", "db.path"]).status.code(), Some(1));
    assert!(!path.exists());

    q.call(&["config", "task.autocreate", "true"]);
    q.call(&["config", "topic.autocreate", "true"]);
    q.call(&["config", "db.path", "folder with spaces/🔥.db"]);
    assert_eq!(q.call(&["config", "db.path"]), "folder with spaces/🔥.db");
    assert_eq!(q.call(&["config", "task.autocreate"]), "true");
    assert!(
        fs::read_to_string(&path)
            .unwrap()
            .contains("autocreate = true")
    );

    let before = fs::read(&path).unwrap();

    for alias in ["list", "--list", "-l"] {
        assert_eq!(
            q.call(&["config", alias]),
            "task.autocreate=true\ntopic.autocreate=true\ndb.path=folder with spaces/🔥.db"
        );
        assert_eq!(fs::read(&path).unwrap(), before);
    }

    let database = q.db.to_str().unwrap();

    for command in [
        vec!["--db", database, "config", "list"],
        vec!["config", "--db", database, "list"],
        vec!["config", "list", "--db", database],
    ] {
        assert_eq!(q.call(&command), q.call(&["config", "--list"]));
        assert!(!q.db.exists());
    }

    for command in [
        vec!["config"],
        vec!["config", "list", "true"],
        vec!["config", "--get", "list"],
        vec!["config", "--unset", "list"],
        vec!["config", "`list`", "true"],
        vec!["config", "list", "--get", "db.path"],
        vec!["config", "list", "--unset", "db.path"],
        vec!["config", "list", "--list"],
        vec!["config", "-l", "list"],
        vec!["config", "task.autoadd", "true"],
        vec!["config", "topic.autoadd", "true"],
        vec!["config", "db.path", ""],
        vec!["config", "unknown", "x"],
        vec!["config", "task.autocreate", "yes"],
        vec!["config", "topic.autocreate", "TRUE"],
        vec!["config", "--get", "task.autocreate", "false"],
        vec!["config", "--unset", "--get", "db.path"],
        vec!["config", "--list", "db.path"],
    ] {
        assert_eq!(q.raw(&command).status.code(), Some(2), "{command:?}");
        assert_eq!(fs::read(&path).unwrap(), before);
    }

    q.call(&["config", "db.path", "list"]);
    assert_eq!(q.call(&["config", "--get", "db.path"]), "list");

    for key in ["task.autocreate", "topic.autocreate", "db.path"] {
        q.call(&["config", "--unset", key]);
    }

    assert_eq!(q.call(&["config", "task.autocreate"]), "false");
    assert_eq!(q.call(&["config", "topic.autocreate"]), "false");
    assert_eq!(
        q.raw(&["config", "--get", "db.path"]).status.code(),
        Some(1)
    );
    assert!(!q.db.exists());
}

#[test]
fn configured_database_priority_and_tilde_expansion_are_observed_by_real_commands() {
    let q = Queue::new();
    q.call(&["config", "db.path", "~/configured/queue.db"]);

    let output = TestProcess::spawn(
        q.cmd(&["topic", "create", "configured"])
            .env_remove("ANYQ_DB")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped()),
    )
    .output();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let configured = q.dir.path().join("configured/queue.db");
    assert!(configured.exists());
    assert!(!q.db.exists());

    q.topic("environment");
    assert_eq!(q.call(&["topic", "list"]), "- environment");

    let explicit = q.dir.path().join("explicit.db");
    let output = TestProcess::spawn(
        q.cmd(&["topic", "create", "explicit"])
            .arg("--db")
            .arg(&explicit)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped()),
    )
    .output();

    assert!(output.status.success());
    assert!(explicit.exists());
    assert_eq!(q.call(&["topic", "list"]), "- environment");
    let output = TestProcess::spawn(
        q.cmd(&["topic", "list"])
            .env_remove("ANYQ_DB")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped()),
    )
    .output();

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim(),
        "- configured"
    );

    q.call(&["config", "--unset", "db.path"]);
    let fallback = q.dir.path().join("data");
    let output = TestProcess::spawn(
        q.cmd(&["topic", "list"])
            .env_remove("ANYQ_DB")
            .env("XDG_DATA_HOME", &fallback)
            .env("LOCALAPPDATA", &fallback)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped()),
    )
    .output();

    assert!(output.status.success());
    assert!(fallback.join("anyq/queue.db").exists());
}

#[test]
fn invalid_config_keeps_its_bytes_and_blocks_queue_commands_but_not_help() {
    let q = Queue::new();
    let path = q.dir.path().join(".anyq/config.toml");
    fs::create_dir_all(path.parent().unwrap()).unwrap();

    for invalid in [
        "[broken",
        "list = true",
        "[task]\nautocreate = 1",
        "[topic]\nautocreate = \"true\"",
        "[db]\npath = \"\"",
        "[task]\nautoadd = true",
    ] {
        fs::write(&path, invalid).unwrap();

        for command in [
            vec!["config", "list"],
            vec!["config", "--list"],
            vec!["config", "task.autocreate", "true"],
            vec!["task", "list"],
        ] {
            assert_eq!(q.raw(&command).status.code(), Some(2));
            assert_eq!(fs::read_to_string(&path).unwrap(), invalid);
        }

        assert!(q.raw(&["help", "config"]).status.success());
        assert!(q.raw(&["--version"]).status.success());
        assert!(!q.db.exists());
    }
}

#[test]
fn independent_autocreate_switches_cover_reads_links_capacity_and_initial_topics() {
    for (tasks, topics) in [(false, false), (true, false), (false, true), (true, true)] {
        let q = Queue::new();
        q.call(&[
            "config",
            "task.autocreate",
            if tasks { "true" } else { "false" },
        ]);
        q.call(&[
            "config",
            "topic.autocreate",
            if topics { "true" } else { "false" },
        ]);

        assert_eq!(q.raw(&["task", "fresh-task"]).status.success(), tasks);
        assert_eq!(q.raw(&["topic", "fresh-topic"]).status.success(), topics);
        assert_eq!(
            q.raw(&["topic", "capacity", "new-capacity", "2"])
                .status
                .success(),
            topics
        );
        assert_eq!(
            q.raw(&["task", "missing-job", "link", "missing-resource"])
                .status
                .success(),
            tasks && topics
        );

        let names = q.call(&["task", "list"]);
        let resources = q.call(&["topic", "list"]);
        assert_eq!(names.contains("missing-job"), tasks && topics);
        assert_eq!(resources.contains("missing-resource"), tasks && topics);
        assert_eq!(
            q.raw(&["task", "create", "explicit", "-t", "initial"])
                .status
                .success(),
            topics
        );
        assert_eq!(q.call(&["task", "list"]).contains("explicit"), topics);
    }
}

#[test]
fn autocreate_rolls_back_failed_wait_complete_and_link_and_preserves_existing_states() {
    let q = Queue::new();
    q.call(&["config", "task.autocreate", "true"]);
    q.call(&["config", "topic.autocreate", "true"]);

    for command in [
        vec!["task", "wait", "missing"],
        vec!["task", "complete", "missing"],
        vec!["task", "link", "missing", "1-invalid"],
        vec![
            "task",
            "create",
            "missing",
            "-t",
            "transient",
            "-t",
            "1-invalid",
        ],
    ] {
        assert_eq!(q.raw(&command).status.code(), Some(2), "{command:?}");
        assert_eq!(q.call(&["task", "list"]), "");
        assert_eq!(q.call(&["topic", "list"]), "");
    }

    q.call(&["task", "missing", "complete", "-f"]);
    assert_eq!(q.state("missing"), "completed");
    assert_eq!(
        q.raw(&["task", "missing", "link", "transient"])
            .status
            .code(),
        Some(2)
    );
    assert_eq!(q.call(&["topic", "list"]), "");

    q.call(&["task", "job", "link", "cpu", "-wc"]);
    assert_eq!(q.state("job"), "completed");
    assert_eq!(q.usage("cpu"), 0);
    assert_eq!(q.raw(&["task", "create", "job"]).status.code(), Some(2));
}

#[test]
fn independent_processes_autocreate_one_pair_of_objects_without_lost_links() {
    let q = Queue::new();
    q.call(&["config", "task.autocreate", "true"]);
    q.call(&["config", "topic.autocreate", "true"]);
    let children: Vec<_> = (0..4)
        .map(|_| {
            TestProcess::spawn(
                q.cmd(&["task", "job", "link", "cpu"])
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped()),
            )
        })
        .collect();

    for child in children {
        let output = child.output();

        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    assert_eq!(q.call(&["task", "list"]), "- job");
    assert_eq!(q.call(&["topic", "list"]), "- cpu");
    assert_eq!(
        q.call(&["task", "job"]),
        "task: job\nstatus: queued\ntopics:\n- cpu"
    );
    q.call(&["task", "job", "wait", "-c"]);

    assert_eq!(q.state("job"), "completed");
    assert_eq!(q.usage("cpu"), 0);
}

#[test]
fn capacity_numeric_boundaries_and_global_database_options_preserve_the_contract() {
    let q = Queue::new();

    for number in ["0", "-1", "4294967296", "1.5"] {
        assert_eq!(
            q.raw(&["topic", "create", "cpu", "-c", number])
                .status
                .code(),
            Some(2)
        );
        assert!(!q.db.exists());
    }

    q.call(&["topic", "create", "cpu", "-c", "4294967295"]);
    assert_eq!(q.call(&["topic", "cpu", "capacity"]), "4294967295");
    let database = q.db.to_str().unwrap();

    q.call(&[
        "--db", database, "task", "job", "--db", database, "create", "-twc", "cpu",
    ]);
    assert_eq!(q.state("job"), "completed");

    q.call(&["topic", "cpu", "--db", database, "capacity", "1"]);
    assert_eq!(q.call(&["topic", "capacity", "cpu"]), "1");
    assert_eq!(q.usage("cpu"), 0);
}
