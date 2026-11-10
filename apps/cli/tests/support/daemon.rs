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
    let stream = endpoint.connect().unwrap();
    let request = RequestFrame {
        version: PROTOCOL_VERSION,
        id: 1,
        request: QueueRequest::Open {
            database_id: endpoint.identity,
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

#[test]
fn status_is_read_only_and_lifecycle_messages_support_both_languages() {
    let queue = Queue::new();

    assert_eq!(queue.call(&["daemon", "status"]), "daemon is stopped");
    assert_eq!(
        queue.call(&["--lang", "ru", "daemon", "status"]),
        "демон остановлен"
    );
    assert!(!queue.db.exists());

    let first = queue.call(&["daemon", "start"]);
    assert!(first.starts_with("daemon is running (pid "));
    assert_eq!(queue.call(&["daemon", "start"]), first);
    assert_eq!(queue.call(&["daemon", "status"]), first);
    assert!(
        queue
            .call(&["daemon", "status", "--lang=ru"])
            .starts_with("демон запущен")
    );

    queue.topic("cpu");
    queue.call(&["--lang", "ru", "task", "localized", "create", "-twc", "cpu"]);
    assert_eq!(queue.state("localized"), "completed");

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
    slow.write_all(&[0, 0]).unwrap();

    let mut oversized = endpoint.connect().unwrap();
    oversized.write_all(&u32::MAX.to_be_bytes()).unwrap();
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
