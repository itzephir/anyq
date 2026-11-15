use crate::support::{
    process::TestProcess,
    queue::{Queue, wait_for_file},
};
use anyq_plugin_wire::{
    CapabilityRequest, CapabilityResult, HostFrame, ManagedWorkDto, PROTOCOL_VERSION, PluginFrame,
    decode, encode,
};
use anyq_queue_api::{Metadata, MetadataValue, Task, TaskState};
use rusqlite::Connection;
use serde_json::{Value, json};
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    process::Stdio,
    sync::mpsc::{self, Receiver},
    thread,
    time::{Duration, Instant},
};

fn wait(mut condition: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(15);

    while !condition() {
        assert!(
            Instant::now() < deadline,
            "shell recovery condition timed out"
        );
        thread::sleep(Duration::from_millis(20));
    }
}

fn status(queue: &Queue, name: &str) -> Value {
    serde_json::from_str(&queue.call(&["plugin", "status", name])).unwrap()
}

fn send(output: &mut impl Write, frame: HostFrame) {
    output.write_all(&encode(&frame).unwrap()).unwrap();
    output.write_all(b"\n").unwrap();
    output.flush().unwrap();
}

fn receive(input: &Receiver<PluginFrame>) -> PluginFrame {
    input.recv_timeout(Duration::from_secs(5)).unwrap()
}

#[test]
fn recovered_completion_retries_the_same_receipt_and_recognizes_a_manual_winner() {
    let queue = Queue::new();
    let mut process = TestProcess::spawn(
        queue
            .cmd(&["plugin-runtime", "shell"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped()),
    );
    let mut input = process.child.as_mut().unwrap().stdin.take().unwrap();
    let output = process.child.as_mut().unwrap().stdout.take().unwrap();
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        for line in BufReader::new(output).lines() {
            let frame = decode::<PluginFrame>(line.unwrap().as_bytes()).unwrap();
            if sender.send(frame).is_err() {
                break;
            }
        }
    });
    send(
        &mut input,
        HostFrame::Hello {
            version: PROTOCOL_VERSION,
            plugin: "shell".into(),
            max_frame_bytes: anyq_plugin_wire::MAX_FRAME_BYTES,
            state: Metadata::default().into(),
        },
    );
    assert!(matches!(receive(&receiver), PluginFrame::Hello { .. }));
    assert!(matches!(
        receive(&receiver),
        PluginFrame::Request {
            request: CapabilityRequest::ManagedList { after: None },
            ..
        }
    ));
    let mut details = Metadata::default();
    details
        .set("autocomplete", MetadataValue::Bool(true))
        .unwrap();
    details
        .set("status", MetadataValue::String("exited".into()))
        .unwrap();

    send(
        &mut input,
        HostFrame::Result {
            id: "recovery".into(),
            result: CapabilityResult::ManagedList {
                work: vec![ManagedWorkDto {
                    details: details.into(),
                    owner: "shell".into(),
                    action_id: "shell:original".into(),
                    task: "owner".into(),
                    state: "stopped".into(),
                    cancellation_requested: false,
                }],
                next: None,
            },
        },
    );
    for attempt in 0..2 {
        let PluginFrame::Request { id, request } = receive(&receiver) else {
            panic!("expected recovered completion");
        };
        assert_eq!(id, "recovered:shell:original");
        assert!(matches!(
            request,
            CapabilityRequest::Complete { action_id, task, force: false }
                if action_id == "complete:shell:original" && task == "owner"
        ));
        send(
            &mut input,
            HostFrame::Result {
                id,
                result: if attempt == 0 {
                    CapabilityResult::CompletionPending
                } else {
                    CapabilityResult::Error {
                        category: "conflict".into(),
                        message: "owner is already completed".into(),
                    }
                },
            },
        );
    }
    let PluginFrame::Request { id, request } = receive(&receiver) else {
        panic!("expected task reconciliation");
    };
    assert!(matches!(request, CapabilityRequest::ReadTask { name } if name == "owner"));
    send(
        &mut input,
        HostFrame::Result {
            id,
            result: CapabilityResult::Task {
                task: Task::restore(1, "owner".into(), TaskState::Completed, vec!["cpu".into()])
                    .unwrap()
                    .into(),
            },
        },
    );

    // Observe past the documented 500 ms retry interval: terminal work must be retired.
    assert!(matches!(
        receiver.recv_timeout(Duration::from_millis(750)),
        Err(mpsc::RecvTimeoutError::Timeout)
    ));
    send(&mut input, HostFrame::Stop);
    drop(input);
    assert!(process.output().status.success());
}

#[test]
fn recovered_autocomplete_retries_after_another_managed_worker_confirms_stopped() {
    let queue = Queue::new();
    let gate = queue.dir.path().join("completion-gate");
    let output = TestProcess::spawn(
        queue
            .cmd(&["daemon", "start"])
            .env("ANYQ_SHELL_TEST_CHECKPOINT", "shell.before-complete")
            .env("ANYQ_SHELL_TEST_DIRECTORY", &gate)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped()),
    )
    .output();
    assert!(output.status.success());
    queue.topic("cpu");
    let blocker = queue.log("blocker");
    queue.call(&[
        "plugin",
        "enable",
        "blocker",
        env!("CARGO_BIN_EXE_anyq-plugin-fixture"),
        "managed-controlled",
        blocker.to_str().unwrap(),
    ]);
    queue.call(&[
        "plugin",
        "enable",
        "shell",
        env!("CARGO_BIN_EXE_anyq"),
        "plugin-runtime",
        "shell",
    ]);
    wait(|| status(&queue, "blocker")["running"] == true);
    wait(|| status(&queue, "shell")["running"] == true);
    let counter = queue.dir.path().join("launches.txt");
    let command = if cfg!(windows) {
        "echo launch>>launches.txt"
    } else {
        "printf 'launch\\n' >> launches.txt"
    };
    let metadata = queue.dir.path().join("metadata.json");
    fs::write(
        &metadata,
        serde_json::to_vec(&json!({
            "shell": {"sh":[command],"cwd":queue.dir.path(),"autocomplete":true}
        }))
        .unwrap(),
    )
    .unwrap();
    queue.call(&[
        "task",
        "create",
        "owner",
        "-t",
        "cpu",
        "--metafile",
        metadata.to_str().unwrap(),
    ]);
    queue.task("next", &["cpu"]);
    wait_for_file(&gate.join("ready"), Some("shell.before-complete"));
    wait_for_file(&blocker.with_extension("marker"), Some("started"));

    fs::write(gate.join("crash"), b"crash after confirmed exit").unwrap();
    wait_for_file(&blocker.with_extension("cancelled"), Some("cancelled"));

    assert_eq!(queue.state("owner"), "running");
    assert_eq!(queue.state("next"), "queued");
    assert_eq!(queue.usage("cpu"), 1);
    fs::write(blocker.with_extension("release"), b"confirmed stop").unwrap();
    wait(|| queue.state("owner") == "completed");
    wait(|| queue.state("next") == "running");

    assert_eq!(fs::read_to_string(counter).unwrap().lines().count(), 1);
    assert_eq!(queue.usage("cpu"), 1);
    let work: Vec<Value> =
        serde_json::from_str(&queue.call(&["plugin", "inspect", "shell"])).unwrap();
    assert_eq!(work.len(), 1);
    assert_eq!(work[0]["state"], "stopped");
    let connection = Connection::open(&queue.db).unwrap();
    let completions: i64 = connection.query_row(
        "SELECT count(*) FROM queue_events WHERE json_extract(payload,'$.kind')='task.completed' AND json_extract(payload,'$.task.name')='owner'",
        [], |row| row.get(0),
    ).unwrap();
    assert_eq!(completions, 1);
}

#[test]
fn live_autocomplete_retires_after_manual_completion_wins() {
    let queue = Queue::new();
    let gate = queue.dir.path().join("live-completion-gate");
    let output = TestProcess::spawn(
        queue
            .cmd(&["daemon", "start"])
            .env(
                "ANYQ_SHELL_TEST_CHECKPOINT",
                "shell.before-complete,shell.execution-finished",
            )
            .env("ANYQ_SHELL_TEST_DIRECTORY", &gate)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped()),
    )
    .output();
    assert!(output.status.success());
    let owner = queue.call(&["daemon", "status"]);
    queue.topic("cpu");
    queue.call(&[
        "plugin",
        "enable",
        "shell",
        env!("CARGO_BIN_EXE_anyq"),
        "plugin-runtime",
        "shell",
    ]);
    wait(|| status(&queue, "shell")["running"] == true);
    let command = if cfg!(windows) {
        "echo launch>>launches.txt"
    } else {
        "printf 'launch\\n' >> launches.txt"
    };
    let metadata = queue.dir.path().join("live-metadata.json");
    fs::write(
        &metadata,
        serde_json::to_vec(&json!({
            "shell": {"sh":[command],"cwd":queue.dir.path(),"autocomplete":true}
        }))
        .unwrap(),
    )
    .unwrap();
    queue.call(&[
        "task",
        "create",
        "owner",
        "-t",
        "cpu",
        "--metafile",
        metadata.to_str().unwrap(),
    ]);
    queue.task("next", &["cpu"]);
    wait_for_file(&gate.join("ready"), Some("shell.before-complete"));

    assert_eq!(queue.state("owner"), "running");
    assert_eq!(queue.state("next"), "queued");
    queue.call(&["task", "owner", "complete"]);

    assert_eq!(queue.state("owner"), "completed");
    assert_eq!(queue.state("next"), "running");
    fs::write(gate.join("release"), b"manual completion won").unwrap();
    wait_for_file(&gate.join("ready"), Some("shell.execution-finished"));

    assert_eq!(queue.call(&["daemon", "status"]), owner);
    assert_eq!(queue.usage("cpu"), 1);
    assert_eq!(queue.state("owner"), "completed");
    assert_eq!(queue.state("next"), "running");
    assert_eq!(
        fs::read_to_string(queue.dir.path().join("launches.txt"))
            .unwrap()
            .lines()
            .count(),
        1
    );
    let plugin = status(&queue, "shell");
    assert_eq!(plugin["running"], true);
    assert_eq!(plugin["failures"], 0);
    assert_eq!(plugin["unresolved_actions"], 0);
    let connection = Connection::open(&queue.db).unwrap();
    let completions: i64 = connection.query_row(
        "SELECT count(*) FROM queue_events WHERE json_extract(payload,'$.kind')='task.completed' AND json_extract(payload,'$.task.name')='owner'",
        [], |row| row.get(0),
    ).unwrap();
    assert_eq!(completions, 1);
}
