use super::*;
use anyq_queue_api::{QueueEventKind, ReadQueueEvents};

#[test]
fn event_identity_payload_and_original_directory_survive_reopen_and_later_topic_edits() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("events.db");
    let native = anyq_queue_api::ExecutionDirectory::from(
        anyq_queue_wire::CallerDirectoryDto::from(directory.path().to_path_buf()),
    );
    struct FixedClock;

    impl anyq_queue_dispatch::ports::Clock for FixedClock {
        fn now_millis(&self) -> i64 {
            777
        }
    }

    let mut store = Sqlite::open(&path)
        .unwrap()
        .with_directory(native.clone())
        .with_clock(std::rc::Rc::new(FixedClock));
    CreateTopicInteractor::new(store.clone())
        .create(Topic::new("cpu".into(), 1).unwrap())
        .unwrap();
    RegisterTaskInteractor::new(store.clone())
        .register(RegisterTaskRequest {
            name: "job".into(),
            topics: vec!["cpu".into()],
            metadata: Metadata::default(),
            start: anyq_queue_api::StartPreference::Default,
        })
        .unwrap();
    let events = store.after(0, 256).unwrap();
    let running = events.last().unwrap().clone();

    assert_eq!(running.snapshot.kind, QueueEventKind::TaskRunning);
    assert_eq!(running.snapshot.occurred_at, 777);
    assert_eq!(running.snapshot.directory, Some(native));
    SetTopicMetadataInteractor::new(store.clone(), QueuePolicy::default())
        .set(SetTopicMetadataRequest {
            topic: "cpu".into(),
            key: "timeout".into(),
            value: MetadataValue::String("1s".into()),
        })
        .unwrap();
    drop(store);
    let mut store = Sqlite::open(&path).unwrap();
    let reread = store.after(0, 256).unwrap();

    assert_eq!(&reread[..events.len()], &events);
    assert_eq!(reread[events.len() - 1].id(), running.id());
    assert_eq!(
        reread[events.len() - 1].snapshot.topics[0].metadata,
        Metadata::default()
    );
    assert!(
        reread
            .windows(2)
            .all(|pair| pair[0].sequence < pair[1].sequence)
    );
    assert_eq!(store.watermark().unwrap(), reread.last().unwrap().sequence);
}

#[test]
fn failed_sql_write_does_not_leave_events_and_upgrade_does_not_invent_old_running_events() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("legacy.db");
    let connection = Connection::open(&path).unwrap();
    connection
        .execute_batch(include_str!("schema_v2.sql"))
        .unwrap();
    drop(connection);
    let mut store = Sqlite::open(&path).unwrap();

    assert!(store.after(0, 256).unwrap().is_empty());
    assert_eq!(task(&store, "owner").state(), TaskState::Running);
    store.connection.borrow().execute_batch("CREATE TRIGGER fail_event BEFORE INSERT ON queue_events BEGIN SELECT RAISE(ABORT,'injected'); END;").unwrap();
    assert!(
        CompleteTaskInteractor::new(store.clone())
            .complete("owner", false)
            .is_err()
    );

    assert!(store.after(0, 256).unwrap().is_empty());
    assert_eq!(task(&store, "owner").state(), TaskState::Running);
    assert_eq!(task(&store, "next").state(), TaskState::Queued);
}

#[test]
fn metadata_schema_four_upgrades_without_rewriting_snapshots_or_existing_ownership() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("metadata-v4.db");
    let connection = Connection::open(&path).unwrap();
    connection
        .execute_batch(include_str!("../schema.sql"))
        .unwrap();
    connection.execute_batch("INSERT INTO topics(name,capacity,metadata) VALUES ('cpu',1,'{\"unknown\":true}'); INSERT INTO tasks(id,name,state,metadata) VALUES (1,'owner','running','{\"n\":9007199254740993}'),(2,'next','queued','{}'); INSERT INTO task_topics VALUES (1,'cpu'),(2,'cpu'); PRAGMA user_version=4;").unwrap();
    drop(connection);

    let mut store = Sqlite::open(&path).unwrap();

    assert_eq!(task(&store, "owner").state(), TaskState::Running);
    assert_eq!(task(&store, "next").state(), TaskState::Queued);
    assert_eq!(
        task(&store, "owner").metadata().get(Some("n")).unwrap(),
        &MetadataValue::Number(anyq_queue_api::MetadataNumber::parse("9007199254740993").unwrap())
    );
    assert_eq!(
        GetTopicInteractor::new(store.clone())
            .get("cpu")
            .unwrap()
            .metadata
            .get(Some("unknown"))
            .unwrap(),
        &MetadataValue::Bool(true)
    );
    assert!(store.after(0, 256).unwrap().is_empty());
    assert_eq!(usage(&store, "cpu"), 1);
}
