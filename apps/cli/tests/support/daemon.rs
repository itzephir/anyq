use crate::support::{large_output, process::TestProcess, queue::Queue};
use anyq_daemon_api::{
    PROTOCOL_VERSION, PolicyDto, QueueRequest, QueueResponse, RequestFrame, ResponseFrame,
};
use anyq_daemon_transport::{Endpoint, FrameReader, Stream, send_frame};
use anyq_queue_api::QueuePolicy;
use std::{
    io::{Read, Write},
    process::Stdio,
    sync::atomic::AtomicBool,
    thread,
    time::{Duration, Instant},
};

fn open_session(queue: &Queue) -> Stream {
    queue.call(&["daemon", "start"]);
    let endpoint = Endpoint::resolve(&queue.db).unwrap();
    open_endpoint(&endpoint)
}

fn open_endpoint(endpoint: &Endpoint) -> Stream {
    let stream = endpoint.connect().unwrap();
    let request = RequestFrame {
        version: PROTOCOL_VERSION,
        id: 1,
        request: QueueRequest::Open {
            caller_directory: std::env::current_dir().unwrap().into(),
            database_id: endpoint.identity.clone(),
            policy: PolicyDto::from(QueuePolicy::default()),
            #[cfg(feature = "integration-test-hooks")]
            gate: anyq_daemon_api::GateDto {
                name: None,
                hit: 1,
                directory: None,
            },
        },
    };
    send_frame(
        &stream,
        &request,
        &AtomicBool::new(false),
        Duration::from_secs(5),
    )
    .unwrap();
    let response: ResponseFrame = FrameReader::default()
        .receive(&stream, &AtomicBool::new(false), Duration::from_secs(5))
        .unwrap();
    assert!(matches!(response.response, QueueResponse::Ready { .. }));

    stream
}

fn write_raw(stream: &mut Stream, mut bytes: &[u8]) {
    let deadline = Instant::now() + Duration::from_secs(5);

    while !bytes.is_empty() {
        match stream.write(bytes) {
            Ok(0) => panic!("connection closed during fixture write"),
            Ok(count) => bytes = &bytes[count..],
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                assert!(Instant::now() < deadline);
                thread::sleep(Duration::from_millis(2));
            }
            Err(error) => panic!("fixture write failed: {error}"),
        }
    }
}

#[test]
fn status_is_read_only_and_owner_lifecycle_is_reusable() {
    let mut queue = Queue::new();
    queue.db = queue.dir.path().join("queue with spaces 🦀.db");

    assert_eq!(queue.call(&["daemon", "status"]), "daemon is stopped");
    assert!(!queue.db.exists());

    let first = queue.call(&["daemon", "start"]);
    assert!(first.starts_with("daemon is running (pid "));
    assert_eq!(queue.call(&["daemon", "start"]), first);
    assert_eq!(queue.call(&["daemon", "status"]), first);
    queue.call(&["daemon", "stop"]);
    assert_eq!(queue.call(&["daemon", "status"]), "daemon is stopped");
}

#[test]
fn stop_restart_and_owner_crash_preserve_states_and_all_resource_slots() {
    let queue = Queue::new();
    let owner = queue.spawn_owner();
    queue.topic("cpu");
    queue.topic("repo");
    queue.task("owner", &["cpu", "repo"]);
    queue.task("next", &["cpu", "repo"]);

    owner.stop();

    assert_eq!(queue.call(&["daemon", "status"]), "daemon is stopped");
    queue.call(&["daemon", "start"]);
    assert_eq!(queue.state("owner"), "running");
    assert_eq!(queue.state("next"), "queued");
    assert_eq!(queue.usage("cpu"), 1);
    assert_eq!(queue.usage("repo"), 1);

    queue.call(&["daemon", "stop"]);
    queue.call(&["daemon", "start"]);
    assert_eq!(queue.state("owner"), "running");
    assert_eq!(queue.state("next"), "queued");
    queue.call(&["task", "complete", "owner"]);
    assert_eq!(queue.state("next"), "running");
}

#[test]
fn concurrent_first_clients_share_one_owner_and_canonical_relative_paths() {
    let queue = Queue::new();
    let mut clients = Vec::new();

    for index in 0..12 {
        clients.push(TestProcess::spawn(
            queue
                .cmd(&["task", "create", &format!("job{index}")])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped()),
        ));
    }

    for client in clients {
        let output = client.output();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    assert_eq!(queue.call(&["task", "list"]).lines().count(), 12);
    let owner = queue.call(&["daemon", "status"]);
    let relative = queue
        .cmd(&["daemon", "status", "--db", "./tasks.db"])
        .current_dir(queue.dir.path())
        .output()
        .unwrap();
    assert!(relative.status.success());
    assert_eq!(String::from_utf8(relative.stdout).unwrap().trim(), owner);
}

#[cfg(unix)]
#[test]
fn symbolic_database_alias_reuses_the_existing_owner() {
    let queue = Queue::new();
    queue.call(&["daemon", "start"]);
    let alias = queue.dir.path().join("alias.db");
    std::os::unix::fs::symlink(&queue.db, &alias).unwrap();

    let output = queue
        .cmd(&["daemon", "status"])
        .arg("--db")
        .arg(alias)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim(),
        queue.call(&["daemon", "status"])
    );
}

#[test]
fn incomplete_oversized_and_incompatible_clients_cannot_block_or_mutate_queue() {
    let queue = Queue::new();
    queue.call(&["daemon", "start"]);
    let endpoint = Endpoint::resolve(&queue.db).unwrap();
    let mut slow = endpoint.connect().unwrap();
    write_raw(&mut slow, &[0, 0]);

    let mut oversized = endpoint.connect().unwrap();
    write_raw(&mut oversized, &u32::MAX.to_be_bytes());
    let malformed = endpoint.connect().unwrap();
    send_frame(
        &malformed,
        &serde_json::json!({"invalid": "frame"}),
        &AtomicBool::new(false),
        Duration::from_secs(5),
    )
    .unwrap();
    let incompatible = endpoint.connect().unwrap();
    send_frame(
        &incompatible,
        &RequestFrame {
            version: PROTOCOL_VERSION + 1,
            id: 1,
            request: QueueRequest::RegisterTask {
                metadata: anyq_queue_api::Metadata::default().into(),
                name: "unexpected".into(),
                topics: vec![],
                start: anyq_daemon_api::StartMode::Default,
            },
        },
        &AtomicBool::new(false),
        Duration::from_secs(5),
    )
    .unwrap();

    assert_eq!(queue.call(&["task", "list"]), "");
    queue.task("valid", &[]);
    assert_eq!(queue.call(&["task", "list"]), "- valid");

    let started = Instant::now();
    queue.call(&["daemon", "stop"]);
    assert!(started.elapsed() < Duration::from_secs(5));
}

#[test]
fn blocked_daemon_response_does_not_block_other_clients_or_shutdown() {
    let queue = Queue::new();
    let size = large_output::seed_tasks(&queue).len();
    assert!(size > 1024 * 1024);
    let mut stream = open_session(&queue);
    send_frame(
        &stream,
        &RequestFrame {
            version: PROTOCOL_VERSION,
            id: 2,
            request: QueueRequest::ListTasks,
        },
        &AtomicBool::new(false),
        Duration::from_secs(5),
    )
    .unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut byte = [0];

    loop {
        match stream.read(&mut byte) {
            Ok(1) => break,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                assert!(Instant::now() < deadline);
                thread::sleep(Duration::from_millis(2));
            }
            result => panic!("expected a response prefix: {result:?}"),
        }
    }

    queue.topic("unrelated");
    let started = Instant::now();
    queue.call(&["daemon", "stop"]);
    assert!(started.elapsed() < Duration::from_secs(5));
}

#[test]
fn concurrent_session_open_close_preserves_responsive_owner_and_slots() {
    let queue = Queue::new();
    queue.topic("cpu");
    queue.task("owner", &["cpu"]);
    queue.task("next", &["cpu"]);

    std::thread::scope(|scope| {
        for _ in 0..4 {
            let queue = &queue;
            scope.spawn(move || {
                for _ in 0..10 {
                    assert_eq!(queue.call(&["task", "list"]), "- next\n- owner");
                }
            });
        }
    });

    assert_eq!(queue.state("owner"), "running");
    assert_eq!(queue.state("next"), "queued");
    assert_eq!(queue.usage("cpu"), 1);
}

fn polling_clients(queue: &Queue, operation: &str, name: &str) -> Vec<TestProcess> {
    let mut clients = Vec::new();

    for index in 0..64 {
        let label = format!("capacity-{index}");
        let mut command = queue.cmd(&["task", operation, name]);
        if operation == "start" {
            command.arg("--wait");
        }

        clients.push(TestProcess::spawn(
            command
                .stdout(Stdio::piped())
                .stderr(std::fs::File::create(queue.log(&label)).unwrap()),
        ));
        queue.wait_for_log(&label, "waiting for");
    }

    for client in &mut clients {
        client.assert_running();
    }

    clients
}

#[test]
fn full_waiting_capacity_preserves_completion_status_and_shutdown() {
    for operation in ["wait", "start"] {
        let queue = Queue::new();
        queue.topic("cpu");
        queue.task("owner", &["cpu"]);
        queue.task("next", &["cpu"]);
        let clients = polling_clients(&queue, operation, "next");

        assert!(
            queue
                .call(&["daemon", "status"])
                .starts_with("daemon is running")
        );
        assert_eq!(queue.state("owner"), "running");
        assert_eq!(queue.state("next"), "queued");
        queue.call(&["task", "wait", "owner"]);
        let overflow = queue.raw(&["task", "wait", "next"]);
        assert!(!overflow.status.success());
        assert!(
            String::from_utf8_lossy(&overflow.stderr)
                .contains("daemon waiting session limit reached")
        );

        queue.topic("independent");
        queue.call(&["task", "create", "immediate", "-t", "independent", "--stop"]);
        queue.call(&["task", "start", "immediate"]);
        assert_eq!(queue.state("immediate"), "running");
        queue.call(&["task", "complete", "immediate"]);

        queue.call(&["task", "complete", "owner"]);

        for client in clients {
            assert!(client.output().status.success());
        }
        assert_eq!(queue.state("next"), "running");
        assert_eq!(queue.usage("cpu"), 1);

        queue.task("tail", &["cpu"]);
        let clients = polling_clients(&queue, operation, "tail");

        queue.call(&["daemon", "stop"]);

        for client in clients {
            assert!(!client.output().status.success());
        }
        assert_eq!(queue.state("next"), "running");
        assert_eq!(queue.state("tail"), "queued");
        assert_eq!(queue.usage("cpu"), 1);
        queue.call(&["daemon", "start"]);
        queue.call(&["task", "complete", "next"]);
        assert_eq!(queue.state("tail"), "running");
    }
}

#[test]
fn full_waiting_capacity_allows_chains_to_reacquire_completion_capacity() {
    let queue = Queue::new();
    queue.topic("cpu");
    queue.task("owner", &["cpu"]);
    let mut clients = Vec::new();

    for index in 0..64 {
        let name = format!("chain{index}");
        clients.push(TestProcess::spawn(
            queue
                .cmd(&["task", "create", &name, "-t", "cpu", "-wc"])
                .stdout(Stdio::piped())
                .stderr(std::fs::File::create(queue.log(&name)).unwrap()),
        ));
        queue.wait_for_log(&name, "waiting for");
    }

    queue.call(&["task", "complete", "owner"]);

    for client in clients {
        let output = client.output();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    for index in 0..64 {
        assert_eq!(queue.state(&format!("chain{index}")), "completed");
    }
    assert_eq!(queue.usage("cpu"), 0);
}

#[test]
fn full_command_capacity_preserves_reserved_lifecycle_access() {
    let queue = Queue::new();
    queue.topic("cpu");
    queue.task("owner", &["cpu"]);
    let endpoint = Endpoint::resolve(&queue.db).unwrap();
    let sessions: Vec<_> = (0..64).map(|_| open_endpoint(&endpoint)).collect();

    assert!(
        queue
            .call(&["daemon", "status"])
            .starts_with("daemon is running")
    );
    let overflow = queue.raw(&["task", "list"]);
    assert!(!overflow.status.success());
    assert!(
        String::from_utf8_lossy(&overflow.stderr).contains("daemon command session limit reached")
    );

    queue.call(&["daemon", "stop"]);

    assert_eq!(queue.state("owner"), "running");
    assert_eq!(queue.usage("cpu"), 1);
    drop(sessions);
}
