use crate::support::queue::Queue;
use anyq_daemon_api::{
    PROTOCOL_VERSION, QueueRequest, QueueResponse, RequestFrame, ResponseFrame, TopicDto,
};
use anyq_daemon_transport::{Endpoint, FrameReader, Stream, send_frame};
use anyq_queue_api::{QueueSnapshot, TaskState};
use std::{sync::atomic::AtomicBool, time::Duration};

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
    queue.call(&["daemon", "start"]);
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
