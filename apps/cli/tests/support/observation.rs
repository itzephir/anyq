use crate::support::queue::Queue;
use anyq_daemon_api::{
    PROTOCOL_VERSION, QueueError, QueueRequest, QueueResponse, RequestFrame, ResponseFrame,
    TopicDto,
};
use anyq_daemon_transport::{Endpoint, FrameReader, Stream, send_frame};
use anyq_queue_api::{QueueSnapshot, TaskState};
use std::{
    io::{self, Read},
    sync::atomic::AtomicBool,
    thread,
    time::{Duration, Instant},
};

fn request(stream: &Stream, id: u64, request: QueueRequest) -> QueueResponse {
    let cancelled = AtomicBool::new(false);
    send_frame(
        stream,
        &RequestFrame {
            version: PROTOCOL_VERSION,
            id,
            request,
        },
        &cancelled,
        Duration::from_secs(5),
    )
    .unwrap();

    let frame: ResponseFrame = FrameReader::for_responses()
        .receive(stream, &cancelled, Duration::from_secs(5))
        .unwrap();
    assert_eq!(frame.version, PROTOCOL_VERSION);
    assert_eq!(frame.id, id);

    frame.response
}

fn observer(queue: &Queue) -> Stream {
    queue.call(&["daemon", "start"]);
    let endpoint = Endpoint::resolve(&queue.db).unwrap();
    open_observer(&endpoint)
}

fn open_observer(endpoint: &Endpoint) -> Stream {
    let stream = endpoint.connect().unwrap();

    let reply = request(
        &stream,
        1,
        QueueRequest::OpenObservation {
            database_id: endpoint.identity.clone(),
        },
    );

    assert!(
        matches!(reply, QueueResponse::Ready { database_id, .. } if database_id == endpoint.identity)
    );
    stream
}

#[test]
fn observation_capacity_cannot_consume_reserved_lifecycle_sessions() {
    let queue = Queue::new();
    queue.topic("cpu");
    queue.task("owner", &["cpu"]);
    let endpoint = Endpoint::resolve(&queue.db).unwrap();
    let sessions: Vec<_> = (0..64).map(|_| open_observer(&endpoint)).collect();

    let overflow = endpoint.connect().unwrap();
    let response = request(
        &overflow,
        1,
        QueueRequest::OpenObservation {
            database_id: endpoint.identity.clone(),
        },
    );

    match response {
        QueueResponse::Error(QueueError::Storage(message)) => {
            assert_eq!(message, "daemon command session limit reached");
        }
        other => panic!("unexpected observation at command capacity: {other:?}"),
    }
    assert!(queue.call(&["daemon", "status"]).contains("running"));

    queue.call(&["daemon", "stop"]);
    drop(sessions);

    assert_eq!(queue.state("owner"), "running");
    assert_eq!(queue.usage("cpu"), 1);
}

#[test]
fn real_owner_observation_is_current_consistent_and_cannot_mutate_even_with_autocreate() {
    let queue = Queue::new();
    queue.call(&["config", "task.autocreate", "true"]);
    queue.call(&["config", "topic.autocreate", "true"]);
    queue.topic("cpu");
    queue.topic("repo");
    queue.topic("empty");
    queue.task("``", &["cpu", "repo"]);
    queue.task("`<b>`queued`</b>`", &["cpu", "repo"]);
    let stream = observer(&queue);
    let expected_id = Endpoint::resolve(&queue.db).unwrap().identity;

    let response = request(
        &stream,
        2,
        QueueRequest::ObserveQueue {
            max_entities: 10_000,
            max_text_bytes: 1024 * 1024,
        },
    );
    let snapshot = match response {
        QueueResponse::Observation {
            database_id,
            snapshot,
        } => {
            assert_eq!(database_id, expected_id);
            QueueSnapshot::try_from(snapshot).unwrap()
        }
        other => panic!("unexpected snapshot: {other:?}"),
    };
    let mutation = request(
        &stream,
        3,
        QueueRequest::CreateTopic {
            topic: TopicDto {
                name: "forbidden".into(),
                capacity: 1,
                autostart: None,
            },
        },
    );
    let oversized = request(
        &stream,
        4,
        QueueRequest::ObserveQueue {
            max_entities: 1,
            max_text_bytes: 1024,
        },
    );

    assert_eq!(snapshot.topics.len(), 3);
    assert_eq!(snapshot.tasks[0].name(), "");
    assert_eq!(snapshot.tasks[1].name(), "<b>`queued`</b>");
    assert_eq!(snapshot.tasks[0].topics(), &["cpu", "repo"]);
    assert_eq!(snapshot.count(TaskState::Running), 1);
    assert_eq!(snapshot.count(TaskState::Queued), 1);
    assert!(matches!(mutation, QueueResponse::Error(_)));
    assert!(matches!(oversized, QueueResponse::Error(_)));
    assert!(!queue.call(&["topic", "list"]).contains("forbidden"));
    assert_eq!(queue.usage("cpu"), 1);
    assert_eq!(queue.usage("repo"), 1);

    queue.call(&["task", "complete", "``"]);
    let response = request(
        &stream,
        5,
        QueueRequest::ObserveQueue {
            max_entities: 10_000,
            max_text_bytes: 1024 * 1024,
        },
    );

    match response {
        QueueResponse::Observation { snapshot, .. } => {
            let snapshot = QueueSnapshot::try_from(snapshot).unwrap();
            assert_eq!(snapshot.tasks.len(), 1);
            assert_eq!(snapshot.tasks[0].name(), "<b>`queued`</b>");
            assert_eq!(snapshot.tasks[0].state(), TaskState::Running);
        }
        other => panic!("unexpected snapshot: {other:?}"),
    }
}

#[test]
fn owner_refuses_missing_existing_only_target_after_discovery() {
    let queue = Queue::new();
    let initial_owner = queue.spawn_owner();
    queue.topic("cpu");
    initial_owner.stop();
    // Discovery can precede opening any SQLite session. Keep the owner alive
    // without a session, after joining the old process, so neither owner-lock
    // release nor Windows SQLite sharing can race the intended removal.
    let _owner = queue.spawn_owner();
    let endpoint = Endpoint::resolve(&queue.db).unwrap();
    std::fs::remove_file(&queue.db).unwrap();
    assert!(!queue.db.exists());
    let stream = endpoint.connect().unwrap();

    let response = request(
        &stream,
        1,
        QueueRequest::OpenObservation {
            database_id: endpoint.identity,
        },
    );

    assert!(matches!(response, QueueResponse::Error(_)));
    assert!(!queue.db.exists());
    assert!(queue.call(&["daemon", "status"]).contains("running"));
}

#[test]
fn observation_rejects_uninitialized_files_without_writes_and_cli_can_initialize_them() {
    for sqlite_header in [false, true] {
        let queue = Queue::new();
        if sqlite_header {
            rusqlite::Connection::open(&queue.db)
                .unwrap()
                .execute_batch("VACUUM")
                .unwrap();
        } else {
            std::fs::write(&queue.db, []).unwrap();
        }
        let original = std::fs::read(&queue.db).unwrap();
        let _owner = queue.spawn_owner();
        let endpoint = Endpoint::resolve(&queue.db).unwrap();
        let stream = endpoint.connect().unwrap();

        let response = request(
            &stream,
            1,
            QueueRequest::OpenObservation {
                database_id: endpoint.identity,
            },
        );

        assert!(matches!(
            response,
            QueueResponse::Error(QueueError::Storage(ref message)) if message.contains("schema is uninitialized")
        ));
        assert_eq!(std::fs::read(&queue.db).unwrap(), original);
        for suffix in ["-wal", "-shm", "-journal"] {
            assert!(!std::path::PathBuf::from(format!("{}{suffix}", queue.db.display())).exists());
        }

        queue.topic("cpu");

        assert!(queue.call(&["topic", "list"]).contains("cpu"));
        let observer = open_observer(&Endpoint::resolve(&queue.db).unwrap());
        assert!(matches!(
            request(
                &observer,
                2,
                QueueRequest::ObserveQueue {
                    max_entities: 10_000,
                    max_text_bytes: 1024 * 1024
                }
            ),
            QueueResponse::Observation { .. }
        ));
    }
}

#[test]
fn observation_rejects_foreign_identity_and_versions_and_accepts_cancellation() {
    let queue = Queue::new();
    queue.topic("cpu");
    queue.task("owner", &["cpu"]);
    let endpoint = Endpoint::resolve(&queue.db).unwrap();
    let foreign = endpoint.connect().unwrap();

    let response = request(
        &foreign,
        1,
        QueueRequest::OpenObservation {
            database_id: format!("{}-foreign", endpoint.identity),
        },
    );

    assert!(matches!(
        response,
        QueueResponse::Error(QueueError::Invalid(_))
    ));

    for (version, operation) in [
        (
            PROTOCOL_VERSION + 1,
            QueueRequest::ObserveQueue {
                max_entities: 10_000,
                max_text_bytes: 1024 * 1024,
            },
        ),
        (PROTOCOL_VERSION, QueueRequest::Cancel),
    ] {
        let stream = open_observer(&endpoint);

        send_frame(
            &stream,
            &RequestFrame {
                version,
                id: 2,
                request: operation,
            },
            &AtomicBool::new(false),
            Duration::from_secs(5),
        )
        .unwrap();
        let result = FrameReader::for_responses().receive::<ResponseFrame>(
            &stream,
            &AtomicBool::new(false),
            Duration::from_secs(5),
        );

        let error = result.expect_err("invalid/cancelled observation must close");
        assert_ne!(error.kind(), io::ErrorKind::TimedOut);
    }

    assert_eq!(queue.state("owner"), "running");
    assert_eq!(queue.usage("cpu"), 1);
    assert_eq!(queue.call(&["task", "list"]), "- owner");
}

#[test]
fn bounded_escaped_observation_and_a_slow_reader_preserve_other_clients_and_shutdown() {
    let queue = Queue::new();
    queue.topic("cpu");
    queue.task("owner", &["cpu"]);
    let mut connection = rusqlite::Connection::open(&queue.db).unwrap();
    let transaction = connection.transaction().unwrap();
    let suffix = "\\\"\n".repeat(1024);
    let mut names = Vec::new();
    for index in 0..256 {
        let name = format!("данные-{index:03}-{suffix}");
        transaction
            .execute(
                "INSERT INTO tasks(name,state) VALUES (?1,'queued')",
                rusqlite::params![name],
            )
            .unwrap();
        transaction
            .execute(
                "INSERT INTO task_topics(task_id,topic_name) VALUES (?1,'cpu')",
                [transaction.last_insert_rowid()],
            )
            .unwrap();
        names.push(name);
    }
    transaction.commit().unwrap();
    drop(connection);
    let stream = observer(&queue);

    let response = request(
        &stream,
        2,
        QueueRequest::ObserveQueue {
            max_entities: 10_000,
            max_text_bytes: 1024 * 1024,
        },
    );

    assert!(serde_json::to_vec(&response).unwrap().len() > 1024 * 1024);
    match response {
        QueueResponse::Observation { snapshot, .. } => {
            let snapshot = QueueSnapshot::try_from(snapshot).unwrap();
            assert_eq!(snapshot.tasks.len(), 257);
            assert_eq!(
                snapshot.tasks[1..]
                    .iter()
                    .map(|task| task.name())
                    .collect::<Vec<_>>(),
                names.iter().map(String::as_str).collect::<Vec<_>>()
            );
        }
        other => panic!("unexpected escaped snapshot: {other:?}"),
    }
    assert!(matches!(
        request(
            &stream,
            3,
            QueueRequest::ObserveQueue {
                max_entities: 10_000,
                max_text_bytes: 1024
            }
        ),
        QueueResponse::Error(QueueError::Invalid(_))
    ));

    send_frame(
        &stream,
        &RequestFrame {
            version: PROTOCOL_VERSION,
            id: 4,
            request: QueueRequest::ObserveQueue {
                max_entities: 10_000,
                max_text_bytes: 1024 * 1024,
            },
        },
        &AtomicBool::new(false),
        Duration::from_secs(5),
    )
    .unwrap();
    let mut stream = stream;
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match stream.read(&mut [0]) {
            Ok(1) => break,
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                assert!(Instant::now() < deadline);
                thread::sleep(Duration::from_millis(2));
            }
            result => panic!("expected an observation response prefix: {result:?}"),
        }
    }

    queue.topic("unrelated");
    assert_eq!(queue.usage("cpu"), 1);
    let stopped = Instant::now();
    queue.call(&["daemon", "stop"]);

    assert!(stopped.elapsed() < Duration::from_secs(5));
    assert_eq!(queue.state("owner"), "running");
    assert_eq!(queue.usage("cpu"), 1);
}
