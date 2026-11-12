use super::{Sqlite, task, usage};
use anyq_queue_api::*;
use anyq_queue_dispatch::*;
use anyq_queue_metadata::{parse_json, to_json};
use rusqlite::Connection;
use std::{
    sync::{Arc, Barrier},
    thread,
};

fn metadata(text: &str) -> Metadata {
    Metadata::new(parse_json(text).unwrap()).unwrap()
}

#[test]
fn metadata_survives_unrelated_mutations_all_states_and_reopening_without_precision_loss() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("queue.db");
    let store = Sqlite::open(&path).unwrap();
    let topic_metadata = metadata(r#"{"policy":{"unknown":true}}"#);
    let task_metadata =
        metadata(r#"{"n":9007199254740993,"array":[1.2300000000000000001,null,false]}"#);
    let mut topic = Topic::new("cpu".into(), 1).unwrap();
    topic.metadata = topic_metadata.clone();
    CreateTopicInteractor::new(store.clone())
        .create(topic)
        .unwrap();
    let created = RegisterTaskInteractor::new(store.clone())
        .register(RegisterTaskRequest {
            name: "job".into(),
            topics: vec!["cpu".into()],
            start: StartPreference::Default,
            metadata: task_metadata.clone(),
        })
        .unwrap();

    assert_eq!(created.metadata(), &task_metadata);
    assert_eq!(created.state(), TaskState::Running);
    SetTopicCapacityInteractor::new(store.clone())
        .set_capacity(Topic::new("cpu".into(), 2).unwrap())
        .unwrap();
    SetTopicAutostartInteractor::new(store.clone())
        .set_autostart("cpu", false)
        .unwrap();
    CompleteTaskInteractor::new(store.clone())
        .complete("job", false)
        .unwrap();
    drop(store);

    let store = Sqlite::open(&path).unwrap();

    assert_eq!(task(&store, "job").metadata(), &task_metadata);
    assert_eq!(task(&store, "job").state(), TaskState::Completed);
    assert_eq!(
        GetTopicInteractor::new(store.clone())
            .get("cpu")
            .unwrap()
            .metadata,
        topic_metadata
    );
    let raw: String = store
        .connection
        .borrow()
        .query_row("SELECT metadata FROM tasks WHERE name='job'", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert!(raw.contains("9007199254740993"));
    assert!(raw.contains("1.2300000000000000001"));
    assert_eq!(usage(&store, "cpu"), 0);
}

#[test]
fn schema_two_and_three_upgrade_atomically_to_empty_metadata_without_new_admission() {
    for version in [2, 3] {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("legacy.db");
        let connection = Connection::open(&path).unwrap();
        connection
            .execute_batch(include_str!("schema_v2.sql"))
            .unwrap();
        if version == 3 {
            connection
                .execute_batch(include_str!("../migrate_v2.sql"))
                .unwrap();
            connection.pragma_update(None, "user_version", 3).unwrap();
        }
        drop(connection);

        let store = Sqlite::open(&path).unwrap();

        assert_eq!(task(&store, "owner").state(), TaskState::Running);
        assert_eq!(task(&store, "next").state(), TaskState::Queued);
        assert_eq!(task(&store, "next").sequence(), 2);
        assert_eq!(task(&store, "owner").metadata(), &Metadata::default());
        assert_eq!(
            GetTopicInteractor::new(store.clone())
                .get("cpu")
                .unwrap()
                .metadata,
            Metadata::default()
        );
        assert_eq!(usage(&store, "cpu"), 1);
        assert_eq!(usage(&store, "repo"), 0);
        let upgraded: i64 = store
            .connection
            .borrow()
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(upgraded, 6);
    }
}

#[test]
fn independent_metadata_patches_cannot_lose_each_other_or_allocate_resources() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("queue.db");
    let store = Sqlite::open(&path).unwrap();
    RegisterTaskInteractor::new(store.clone())
        .register(RegisterTaskRequest {
            name: "job".into(),
            topics: vec![],
            start: StartPreference::Default,
            metadata: Metadata::default(),
        })
        .unwrap();
    let barrier = Arc::new(Barrier::new(8));
    let mut threads = Vec::new();

    for index in 0..8 {
        let path = path.clone();
        let barrier = barrier.clone();
        threads.push(thread::spawn(move || {
            let store = Sqlite::open(&path).unwrap();
            barrier.wait();
            SetTaskMetadataInteractor::new(store, QueuePolicy::default())
                .set(SetTaskMetadataRequest {
                    task: "job".into(),
                    key: format!("keys.k{index}"),
                    value: MetadataValue::Bool(true),
                })
                .unwrap();
        }));
    }
    for thread in threads {
        thread.join().unwrap();
    }

    let task = task(&store, "job");
    assert_eq!(task.state(), TaskState::Registered);
    assert!(task.topics().is_empty());
    for index in 0..8 {
        assert_eq!(
            task.metadata()
                .get(Some(&format!("keys.k{index}")))
                .unwrap(),
            &MetadataValue::Bool(true)
        );
    }
}

#[test]
fn metadata_constraints_and_actual_sql_failures_preserve_existing_data_and_autocreate() {
    let directory = tempfile::tempdir().unwrap();
    let store = Sqlite::open(&directory.path().join("queue.db")).unwrap();
    let policy = QueuePolicy {
        autocreate: AutoCreatePolicy {
            tasks: true,
            topics: true,
        },
        ..QueuePolicy::default()
    };
    store.connection.borrow().execute_batch("CREATE TRIGGER fail_task_meta AFTER UPDATE OF metadata ON tasks BEGIN SELECT RAISE(ABORT,'injected'); END; CREATE TRIGGER fail_topic_meta AFTER UPDATE OF metadata ON topics BEGIN SELECT RAISE(ABORT,'injected'); END;").unwrap();

    assert!(
        SetTaskMetadataInteractor::new(store.clone(), policy)
            .set(SetTaskMetadataRequest {
                task: "missing".into(),
                key: "timeout".into(),
                value: MetadataValue::String("2h".into()),
            })
            .is_err()
    );
    assert!(
        SetTopicMetadataInteractor::new(store.clone(), policy)
            .set(SetTopicMetadataRequest {
                topic: "missing".into(),
                key: "timeout".into(),
                value: MetadataValue::String("2h".into()),
            })
            .is_err()
    );
    assert!(
        ListTasksInteractor::new(store.clone())
            .list()
            .unwrap()
            .is_empty()
    );
    assert!(
        ListTopicsInteractor::new(store.clone())
            .list()
            .unwrap()
            .is_empty()
    );
    store.connection.borrow().execute_batch("DROP TRIGGER fail_task_meta; DROP TRIGGER fail_topic_meta; INSERT INTO tasks(name,state) VALUES ('job','registered');").unwrap();

    for invalid in ["[]", "null", "broken"] {
        assert!(
            store
                .connection
                .borrow()
                .execute("UPDATE tasks SET metadata=?1 WHERE name='job'", [invalid])
                .is_err()
        );
    }
    assert_eq!(
        to_json(task(&store, "job").metadata().value()).unwrap(),
        "{}"
    );
}
