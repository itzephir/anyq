use crate::support::{
    process::TestProcess,
    queue::{Queue, wait_for_file},
};
use anyq_queue_api::{Metadata, MetadataValue};
use serde_json::{Value, json};
use std::{
    fs,
    net::TcpListener,
    path::{Path, PathBuf},
    process::Stdio,
    thread,
    time::{Duration, Instant},
};

fn wait(mut predicate: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(15);
    while !predicate() {
        assert!(Instant::now() < deadline, "shell condition timed out");
        thread::sleep(Duration::from_millis(20));
    }
}

fn enable(queue: &Queue, name: &str) {
    queue.call(&[
        "plugin",
        "enable",
        name,
        env!("CARGO_BIN_EXE_anyq"),
        "plugin-runtime",
        name,
    ]);
    wait(|| status(queue, name)["running"] == true);
}

fn status(queue: &Queue, name: &str) -> Value {
    serde_json::from_str(&queue.call(&["plugin", "status", name])).unwrap()
}

fn inspect(queue: &Queue) -> Vec<Value> {
    serde_json::from_str(&queue.call(&["plugin", "inspect", "shell"])).unwrap()
}

fn details(work: &Value) -> Metadata {
    let dto: anyq_queue_wire::MetadataDto =
        serde_json::from_value(work["details"].clone()).unwrap();
    dto.try_into().unwrap()
}

fn stopped(queue: &Queue) -> bool {
    inspect(queue)
        .first()
        .is_some_and(|work| work["state"] == "stopped")
}

fn file(queue: &Queue, name: &str, value: Value) -> PathBuf {
    let path = queue.dir.path().join(name);
    fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
    path
}

fn plan(queue: &Queue, config: &Path, autocomplete: bool) -> Value {
    json!({"src":[env!("CARGO_BIN_EXE_anyq-plugin-fixture")],"env":{"WORKER_CONFIG":config},"cwd":queue.dir.path(),"autocomplete":autocomplete})
}

fn create(queue: &Queue, name: &str, shell: Value, timeout: Option<&str>) {
    let mut metadata = json!({"shell":shell});
    if let Some(timeout) = timeout {
        metadata["timeout"] = json!(timeout);
    }
    let path = file(queue, &format!("{name}-meta.json"), metadata);
    queue.call(&[
        "task",
        "create",
        name,
        "-t",
        "cpu",
        "--metafile",
        path.to_str().unwrap(),
    ]);
}

fn port(marker: &Path) -> u16 {
    wait_for_file(&marker.with_extension("port"), None);
    fs::read_to_string(marker.with_extension("port"))
        .unwrap()
        .parse()
        .unwrap()
}

#[test]
fn nonzero_outcomes_bounded_logs_and_autocomplete_policy_survive_restart() {
    for autocomplete in [false, true] {
        let queue = Queue::new();
        queue.topic("cpu");
        enable(&queue, "shell");
        let marker = queue.log("work");
        let config = file(
            &queue,
            "work.json",
            json!({"marker":marker,"code":7,"output_bytes":2*1024*1024}),
        );
        create(&queue, "owner", plan(&queue, &config, autocomplete), None);
        queue.task("next", &["cpu"]);
        wait(|| stopped(&queue));
        if autocomplete {
            wait(|| queue.state("owner") == "completed");
        } else {
            assert_eq!(queue.state("owner"), "running");
        }
        let saved = details(&inspect(&queue)[0]);
        assert_eq!(
            saved.get(Some("status")).unwrap(),
            &MetadataValue::String("exited".into())
        );
        let MetadataValue::Array(outcomes) = saved.get(Some("outcomes")).unwrap() else {
            panic!()
        };
        let outcome = Metadata::new(outcomes[0].clone()).unwrap();
        assert_eq!(
            outcome.get(Some("code")).unwrap(),
            &MetadataValue::Number(anyq_queue_api::MetadataNumber::parse("7").unwrap())
        );
        assert_eq!(
            outcome.get(Some("output.stdout_truncated")).unwrap(),
            &MetadataValue::Bool(true)
        );
        let MetadataValue::String(stdout) = outcome.get(Some("output.stdout")).unwrap() else {
            panic!()
        };
        assert!(stdout.len() <= 65536);

        queue.call(&["daemon", "stop"]);
        queue.call(&["daemon", "start"]);

        assert_eq!(details(&inspect(&queue)[0]), saved);
        assert_eq!(
            queue.state("next"),
            if autocomplete { "running" } else { "queued" }
        );
    }
}

#[test]
fn sequential_commands_stop_after_first_failure_and_native_launch_failure_is_recorded() {
    let queue = Queue::new();
    queue.topic("cpu");
    enable(&queue, "shell");
    let commands = if cfg!(windows) {
        vec!["echo first>order.txt", "exit /b 7", "echo third>>order.txt"]
    } else {
        vec![
            "printf first > order.txt",
            "exit 7",
            "printf third >> order.txt",
        ]
    };
    create(
        &queue,
        "owner",
        json!({"sh":commands,"cwd":queue.dir.path(),"autocomplete":true}),
        None,
    );
    wait(|| queue.state("owner") == "completed");
    let saved = details(&inspect(&queue)[0]);
    let MetadataValue::Array(outcomes) = saved.get(Some("outcomes")).unwrap() else {
        panic!()
    };
    assert_eq!(outcomes.len(), 2, "prepared shell outcome: {saved:?}");
    assert_eq!(
        fs::read_to_string(queue.dir.path().join("order.txt"))
            .unwrap()
            .trim(),
        "first"
    );

    create(
        &queue,
        "missing",
        json!({"src":["/definitely-unavailable-anyq-program"],"cwd":queue.dir.path(),"autocomplete":true}),
        None,
    );
    wait(|| queue.state("missing") == "completed");
    let work = inspect(&queue)
        .into_iter()
        .find(|work| work["task"] == "missing")
        .unwrap();

    assert_eq!(
        details(&work).get(Some("status")).unwrap(),
        &MetadataValue::String("launch-failed".into())
    );
}

#[test]
fn delayed_relative_src_uses_captured_client_origin() {
    let queue = Queue::new();
    queue.topic("cpu");
    enable(&queue, "shell");
    let project = queue.dir.path().join("project with spaces");
    fs::create_dir(&project).unwrap();
    let executable = project.join(if cfg!(windows) {
        "worker.exe"
    } else {
        "worker"
    });
    fs::copy(env!("CARGO_BIN_EXE_anyq-plugin-fixture"), &executable).unwrap();
    let marker = queue.log("origin");
    let config = file(&queue, "origin.json", json!({"marker":marker}));
    let metadata = file(
        &queue,
        "origin-meta.json",
        json!({"shell":{"src":[executable.file_name().unwrap().to_str().unwrap()],"env":{"WORKER_CONFIG":config}}}),
    );
    let result = TestProcess::spawn(
        queue
            .cmd(&[
                "task",
                "create",
                "delayed",
                "--metafile",
                metadata.to_str().unwrap(),
            ])
            .current_dir(&project)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped()),
    )
    .output();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );

    queue.call(&["task", "link", "delayed", "cpu"]);
    wait(|| stopped(&queue));

    assert_eq!(
        PathBuf::from(fs::read_to_string(marker.with_extension("cwd")).unwrap())
            .canonicalize()
            .unwrap(),
        project.canonicalize().unwrap()
    );
    assert_eq!(queue.state("delayed"), "running");
}

#[test]
fn topic_commands_and_invalid_task_plans_never_launch_work() {
    let queue = Queue::new();
    queue.topic("cpu");
    enable(&queue, "shell");
    let marker = queue.log("never");
    let config = file(&queue, "never.json", json!({"marker":marker}));
    let topic = file(
        &queue,
        "topic.json",
        json!({"shell":plan(&queue,&config,true)}),
    );
    queue.call(&["topic", "cpu", "metafile", topic.to_str().unwrap()]);
    queue.task("ordinary", &["cpu"]);
    wait(|| status(&queue, "shell")["lag"] == 0);
    assert!(inspect(&queue).is_empty());
    assert!(!marker.exists());
    queue.call(&["task", "complete", "ordinary"]);

    create(
        &queue,
        "invalid",
        json!({"sh":["echo bad"],"src":["bad"]}),
        None,
    );
    wait(|| status(&queue, "shell")["lag"] == 0);

    assert!(inspect(&queue).is_empty());
    assert!(!marker.exists());
    assert_eq!(queue.state("invalid"), "running");
    assert!(
        queue
            .call(&["plugin", "state", "shell"])
            .contains("exactly one")
    );
}

#[test]
fn descendants_outliving_the_leader_stop_before_manual_disable_or_shutdown_cleanup() {
    for action in ["manual", "disable", "shutdown"] {
        let queue = Queue::new();
        queue.topic("cpu");
        enable(&queue, "shell");
        let marker = queue.log("tree");
        let config = file(
            &queue,
            "tree.json",
            json!({"marker":marker,"descendant":true,"leader_exits":true,"chain_depth":8}),
        );
        let mut execution = plan(&queue, &config, true);
        execution["src"]
            .as_array_mut()
            .unwrap()
            .push(json!("/must-not-launch-after-cancel"));
        create(&queue, "owner", execution, None);
        let port = port(&marker);
        assert!(TcpListener::bind(("127.0.0.1", port)).is_err());
        queue.task("next", &["cpu"]);
        assert_eq!(queue.state("next"), "queued");

        match action {
            "manual" => {
                let result = queue.raw(&["task", "complete", "owner"]);
                assert!(
                    result.status.success(),
                    "manual tree cleanup: {}; work={:?}",
                    String::from_utf8_lossy(&result.stderr),
                    inspect(&queue)
                );
            }
            "disable" => {
                queue.call(&["plugin", "disable", "shell"]);
                wait(|| stopped(&queue));
            }
            _ => {
                queue.call(&["daemon", "stop"]);
                queue.call(&["daemon", "start"]);
            }
        }
        wait(|| TcpListener::bind(("127.0.0.1", port)).is_ok());
        if action != "manual" {
            assert_eq!(queue.state("owner"), "running");
            assert_eq!(queue.state("next"), "queued");
            queue.call(&["task", "complete", "owner"]);
        }

        assert_eq!(queue.state("next"), "running");
        assert_eq!(queue.usage("cpu"), 1);
    }
}

#[test]
fn timeout_stops_the_entire_shell_tree_before_resource_reuse() {
    let queue = Queue::new();
    let clock = queue.dir.path().join("clock");
    fs::write(&clock, "0").unwrap();
    let result = TestProcess::spawn(
        queue
            .cmd(&["daemon", "start"])
            .env("ANYQ_TEST_CLOCK_FILE", &clock)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped()),
    )
    .output();
    assert!(result.status.success());
    queue.topic("cpu");
    enable(&queue, "timeout");
    enable(&queue, "shell");
    let marker = queue.log("timeout-tree");
    let config = file(
        &queue,
        "timeout-tree.json",
        json!({"marker":marker,"descendant":true}),
    );
    create(&queue, "owner", plan(&queue, &config, false), Some("10ms"));
    let port = port(&marker);
    queue.task("next", &["cpu"]);
    wait(|| status(&queue, "timeout")["active_timers"] == 1);
    let next = clock.with_extension("next");
    fs::write(&next, "11").unwrap();

    fs::rename(next, &clock).unwrap();
    wait(|| queue.state("owner") == "completed");

    assert!(TcpListener::bind(("127.0.0.1", port)).is_ok());
    assert_eq!(queue.state("next"), "running");
    assert_eq!(inspect(&queue)[0]["state"], "stopped");
}

#[test]
fn plugin_crash_before_or_after_spawn_never_relaunches_or_releases_resources() {
    for stage in ["shell.before-spawn", "shell.after-spawn"] {
        let queue = Queue::new();
        let gate = queue.dir.path().join("shell-gate");
        let output = TestProcess::spawn(
            queue
                .cmd(&["daemon", "start"])
                .env("ANYQ_SHELL_TEST_CHECKPOINT", stage)
                .env("ANYQ_SHELL_TEST_DIRECTORY", &gate)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped()),
        )
        .output();
        assert!(output.status.success());
        queue.topic("cpu");
        enable(&queue, "shell");
        let marker = queue.log("crash");
        let config = file(
            &queue,
            "crash.json",
            json!({"marker":marker,"descendant":true}),
        );
        let mut execution = plan(&queue, &config, true);
        execution["src"]
            .as_array_mut()
            .unwrap()
            .push(json!("/must-not-launch-after-cancel"));
        create(&queue, "owner", execution, None);
        queue.task("next", &["cpu"]);
        wait_for_file(&gate.join("ready"), Some(stage));
        let held_port = if stage == "shell.after-spawn" {
            wait_for_file(&marker, Some("started"));
            let held_port = port(&marker);
            assert!(TcpListener::bind(("127.0.0.1", held_port)).is_err());
            Some(held_port)
        } else {
            None
        };

        fs::write(gate.join("crash"), b"crash").unwrap();
        wait(|| inspect(&queue)[0]["state"] == "uncertain");
        wait(|| status(&queue, "shell")["running"] == true);
        assert_eq!(inspect(&queue).len(), 1);
        assert_eq!(queue.state("owner"), "running");
        assert_eq!(queue.state("next"), "queued");
        assert_eq!(queue.usage("cpu"), 1);
        if stage == "shell.before-spawn" {
            assert!(!marker.exists());
        }
        fs::write(marker.with_extension("release"), b"independent cleanup").unwrap();
        // The private anchor now terminates its group on plugin-pipe EOF.
        // Independently verify the actual held resource is released below.
        if let Some(held_port) = held_port {
            wait(|| TcpListener::bind(("127.0.0.1", held_port)).is_ok());
        }
        queue.call(&["plugin", "disable", "shell"]);
        let work = inspect(&queue).remove(0);
        let action = work["action_id"].as_str().unwrap();
        queue.call(&["plugin", "resolve", "shell", action, "--confirmed-cleanup"]);
        queue.call(&["task", "complete", "owner"]);

        assert_eq!(queue.state("next"), "running");
    }
}

#[test]
fn owner_crash_preserves_uncertain_shell_receipt_and_never_reuses_live_resources() {
    let queue = Queue::new();
    let owner = queue.spawn_owner();
    queue.topic("cpu");
    enable(&queue, "shell");
    let marker = queue.log("owner-crash");
    let config = file(
        &queue,
        "owner-crash.json",
        json!({"marker":marker,"descendant":true}),
    );
    create(&queue, "owner", plan(&queue, &config, true), None);
    let held_port = port(&marker);
    queue.task("next", &["cpu"]);

    owner.stop();
    queue.call(&["daemon", "start"]);
    wait(|| inspect(&queue)[0]["state"] == "uncertain");

    assert_eq!(queue.state("owner"), "running");
    assert_eq!(queue.state("next"), "queued");
    assert_eq!(inspect(&queue).len(), 1);
    fs::write(marker.with_extension("release"), b"cleanup").unwrap();
    wait(|| TcpListener::bind(("127.0.0.1", held_port)).is_ok());
    queue.call(&["plugin", "disable", "shell"]);
    let work = inspect(&queue).remove(0);
    queue.call(&[
        "plugin",
        "resolve",
        "shell",
        work["action_id"].as_str().unwrap(),
        "--confirmed-cleanup",
    ]);
    queue.call(&["task", "complete", "owner"]);

    assert_eq!(queue.state("next"), "running");
}

#[test]
fn timeout_winning_before_managed_registration_prevents_launch_without_poisoning_delivery() {
    let queue = Queue::new();
    let gate = queue.dir.path().join("registration-gate");
    let clock = queue.dir.path().join("clock");
    fs::write(&clock, "0").unwrap();
    let output = TestProcess::spawn(
        queue
            .cmd(&["daemon", "start"])
            .env("ANYQ_TEST_CLOCK_FILE", &clock)
            .env("ANYQ_SHELL_TEST_CHECKPOINT", "shell.before-register")
            .env("ANYQ_SHELL_TEST_DIRECTORY", &gate)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped()),
    )
    .output();
    assert!(output.status.success());
    queue.topic("cpu");
    enable(&queue, "timeout");
    enable(&queue, "shell");
    let marker = queue.log("prevented");
    let config = file(&queue, "prevented.json", json!({"marker":marker}));
    create(&queue, "owner", plan(&queue, &config, true), Some("10ms"));
    wait_for_file(&gate.join("ready"), Some("shell.before-register"));
    wait(|| status(&queue, "timeout")["active_timers"] == 1);
    let next = clock.with_extension("next");
    fs::write(&next, "11").unwrap();

    fs::rename(next, &clock).unwrap();
    wait(|| queue.state("owner") == "completed");
    fs::write(gate.join("release"), b"continue").unwrap();
    queue.task("next", &["cpu"]);
    wait(|| status(&queue, "shell")["lag"] == 0);

    assert!(!marker.exists());
    assert!(inspect(&queue).is_empty());
    assert_eq!(status(&queue, "shell")["failures"], 0);
    assert_eq!(queue.state("next"), "running");
}
