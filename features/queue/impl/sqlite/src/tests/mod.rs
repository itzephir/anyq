use crate::Sqlite;
use anyq_queue_api::*;
use anyq_queue_dispatch::{
    ports::{Store, TaskFilter},
    *,
};
use rusqlite::Connection;
use std::{
    path::Path,
    sync::{Arc, Barrier},
    thread,
};

fn fixture() -> (tempfile::TempDir, Sqlite) {
    let dir = tempfile::tempdir().unwrap();
    let store = Sqlite::open(&dir.path().join("queue.db")).unwrap();

    (dir, store)
}

fn topic(store: &Sqlite, name: &str, capacity: u32) {
    CreateTopicInteractor::new(store.clone())
        .create(Topic::new(name.into(), capacity).unwrap())
        .unwrap();
}

fn register(store: &Sqlite, name: &str) {
    RegisterTaskInteractor::new(store.clone())
        .register(RegisterTaskRequest {
            metadata: Default::default(),
            start: anyq_queue_api::StartPreference::Default,
            name: name.into(),
            topics: Vec::new(),
        })
        .unwrap();
}

fn add(store: &Sqlite, name: &str, resource: &str) {
    AddTaskToTopicInteractor::new(store.clone())
        .add(name, resource)
        .unwrap();
}

fn check(store: &Sqlite, name: &str) -> Result<bool> {
    CheckTaskRunningInteractor::new(store.clone()).check_running(name)
}

fn complete(store: &Sqlite, name: &str, force: bool) {
    CompleteTaskInteractor::new(store.clone())
        .complete(name, force)
        .unwrap();
}

fn task(store: &Sqlite, name: &str) -> Task {
    store
        .clone()
        .transaction(|records| Ok(records.tasks().task(name)?.unwrap()))
        .unwrap()
}

fn usage(store: &Sqlite, resource: &str) -> u64 {
    store
        .clone()
        .transaction(|records| {
            records.tasks().count_tasks(&TaskFilter {
                topic: Some(resource.into()),
                state: TaskState::Running,
                before_sequence: None,
            })
        })
        .unwrap()
}

#[test]
fn canonical_names_survive_reopening_without_parsing_backticks_twice() {
    let (dir, store) = fixture();
    topic(&store, "``resource``", 1);
    RegisterTaskInteractor::new(store.clone())
        .register(RegisterTaskRequest {
            metadata: Default::default(),
            start: anyq_queue_api::StartPreference::Default,
            name: "``job``".into(),
            topics: vec!["``resource``".into()],
        })
        .unwrap();
    check(&store, "``job``").unwrap();

    drop(store);
    let store = Sqlite::open(&dir.path().join("queue.db")).unwrap();
    let queued = GetTopicQueueInteractor::new(store.clone())
        .get("``resource``")
        .unwrap();
    let restored = GetTaskInteractor::new(store.clone())
        .get("``job``")
        .unwrap();

    assert_eq!(queued.topic.name, "`resource`");
    assert_eq!(queued.tasks.as_slice(), std::slice::from_ref(&restored));
    assert_eq!(restored.name(), "`job`");
    assert_eq!(restored.topics(), &["`resource`"]);
    assert_eq!(restored.state(), TaskState::Running);

    SetTopicCapacityInteractor::new(store.clone())
        .set_capacity(Topic::new("``resource``".into(), 2).unwrap())
        .unwrap();
    complete(&store, "``job``", false);

    assert_eq!(
        GetTopicInteractor::new(store.clone())
            .get("``resource``")
            .unwrap()
            .capacity,
        2
    );
    assert_eq!(usage(&store, "`resource`"), 0);
}

#[test]
fn topic_storage_validates_capacity_and_preserves_canonical_keyword_names() {
    let (_dir, store) = fixture();

    let result = store.clone().transaction(|records| {
        records.topics().put_topic(&Topic {
            metadata: Default::default(),
            name: "cpu".into(),
            capacity: 0,
            autostart: None,
        })
    });

    assert!(result.is_err());
    assert!(
        ListTopicsInteractor::new(store.clone())
            .list()
            .unwrap()
            .is_empty()
    );

    store
        .clone()
        .transaction(|records| records.topics().put_topic(&Topic::new("`list`".into(), 1)?))
        .unwrap();

    assert_eq!(
        ListTopicsInteractor::new(store.clone()).list().unwrap()[0].name,
        "list"
    );

    store
        .connection
        .borrow()
        .execute_batch(
            "PRAGMA ignore_check_constraints=ON; INSERT INTO topics(name,capacity) VALUES ('invalid', 0)",
        )
        .unwrap();

    assert!(ListTopicsInteractor::new(store.clone()).list().is_err());
}

#[test]
fn topic_queue_reads_exact_filters_and_registration_order_without_releasing_slots() {
    let (_dir, store) = fixture();
    topic(&store, "cpu", 1);
    topic(&store, "other", 1);
    register(&store, "z-running");
    add(&store, "z-running", "cpu");
    check(&store, "z-running").unwrap();
    register(&store, "a-queued");
    add(&store, "a-queued", "cpu");
    register(&store, "completed");
    add(&store, "completed", "cpu");
    complete(&store, "completed", true);
    register(&store, "unrelated");
    add(&store, "unrelated", "other");
    register(&store, "unlinked");

    let queue = GetTopicQueueInteractor::new(store.clone())
        .get("cpu")
        .unwrap();
    let before = task(&store, "a-queued").sequence();
    let earlier = store
        .clone()
        .transaction(|records| {
            records.tasks().tasks(&TaskFilter {
                topic: Some("cpu".into()),
                state: TaskState::Queued,
                before_sequence: Some(before),
            })
        })
        .unwrap();

    assert_eq!(
        queue.tasks.iter().map(Task::name).collect::<Vec<_>>(),
        ["z-running", "a-queued"]
    );
    assert!(earlier.is_empty());
    assert_eq!(usage(&store, "cpu"), 1);
    assert_eq!(task(&store, "a-queued").state(), TaskState::Queued);
}

#[test]
fn failed_initial_membership_rolls_back_registration_and_all_links() {
    let (_dir, store) = fixture();
    topic(&store, "cpu", 1);
    topic(&store, "repo", 1);
    store
        .connection
        .borrow()
        .execute_batch(
            "CREATE TRIGGER fail_link BEFORE INSERT ON task_topics
         WHEN NEW.topic_name='repo' BEGIN SELECT RAISE(ABORT,'injected'); END;",
        )
        .unwrap();
    let request = RegisterTaskRequest {
        metadata: Default::default(),
        start: anyq_queue_api::StartPreference::Default,
        name: "build".into(),
        topics: vec!["cpu".into(), "repo".into()],
    };

    let result = RegisterTaskInteractor::new(store.clone()).register(request.clone());

    assert!(result.is_err());
    assert!(
        ListTasksInteractor::new(store.clone())
            .list()
            .unwrap()
            .is_empty()
    );
    let links: i64 = store
        .connection
        .borrow()
        .query_row("SELECT count(*) FROM task_topics", [], |row| row.get(0))
        .unwrap();
    assert_eq!(links, 0);

    store
        .connection
        .borrow()
        .execute_batch("DROP TRIGGER fail_link")
        .unwrap();
    let created = RegisterTaskInteractor::new(store.clone())
        .register(request)
        .unwrap();

    assert_eq!(created.state(), TaskState::Running);
    assert_eq!(created.topics(), &["cpu", "repo"]);
    assert_eq!(usage(&store, "cpu"), 1);
    assert_eq!(usage(&store, "repo"), 1);
    assert!(check(&store, "build").unwrap());
}

#[test]
fn every_lifecycle_state_and_global_order_survive_reopening() {
    let (dir, store) = fixture();
    topic(&store, "cpu", 1);
    topic(&store, "repo", 1);
    register(&store, "done");
    AddTaskToTopicsInteractor::new(store.clone())
        .add(AddTaskToTopicsRequest {
            task: "done".into(),
            topics: vec!["cpu".into(), "repo".into()],
        })
        .unwrap();
    check(&store, "done").unwrap();
    complete(&store, "done", false);
    register(&store, "registered");
    register(&store, "running");
    add(&store, "running", "cpu");
    register(&store, "queued");
    add(&store, "queued", "cpu");
    check(&store, "running").unwrap();
    let before: Vec<_> = ["done", "registered", "queued", "running"]
        .map(|name| task(&store, name))
        .into();

    drop(store);
    let reopened = Sqlite::open(&dir.path().join("queue.db")).unwrap();
    let after: Vec<_> = ["done", "registered", "queued", "running"]
        .map(|name| task(&reopened, name))
        .into();

    assert_eq!(before, after);
    assert_eq!(
        after.iter().map(Task::state).collect::<Vec<_>>(),
        [
            TaskState::Completed,
            TaskState::Registered,
            TaskState::Queued,
            TaskState::Running
        ]
    );
    assert_eq!(usage(&reopened, "cpu"), 1);
    assert_eq!(usage(&reopened, "repo"), 0);
    assert!(
        RegisterTaskInteractor::new(reopened.clone())
            .register(RegisterTaskRequest {
                metadata: Default::default(),
                start: anyq_queue_api::StartPreference::Default,
                name: "done".into(),
                topics: Vec::new()
            })
            .is_err()
    );
}

#[test]
fn sqlite_counts_all_memberships_and_obeys_global_order() {
    let (_dir, store) = fixture();
    topic(&store, "cpu", 1);
    topic(&store, "repo", 1);
    register(&store, "owner");
    add(&store, "owner", "cpu");
    register(&store, "a");
    register(&store, "b");
    add(&store, "a", "cpu");
    add(&store, "b", "cpu");
    add(&store, "b", "repo");
    add(&store, "a", "repo");
    add(&store, "a", "cpu");

    assert!(!check(&store, "b").unwrap());
    assert_eq!(usage(&store, "repo"), 0);

    complete(&store, "owner", false);

    assert_eq!(task(&store, "a").state(), TaskState::Running);
    assert_eq!(task(&store, "b").state(), TaskState::Queued);
    assert_eq!(usage(&store, "cpu"), 1);
    assert_eq!(usage(&store, "repo"), 1);

    complete(&store, "a", false);

    assert_eq!(task(&store, "b").state(), TaskState::Running);
    assert_eq!(task(&store, "b").topics(), &["cpu", "repo"]);
    assert!(check(&store, "b").unwrap());
    assert!(check(&store, "b").unwrap());
}

#[test]
fn capacity_reduction_and_partial_admission_are_rejected() {
    let (_dir, store) = fixture();
    topic(&store, "cpu", 2);
    topic(&store, "repo", 1);

    for name in ["a", "b"] {
        register(&store, name);
        add(&store, name, "cpu");
        check(&store, name).unwrap();
    }

    assert!(
        SetTopicCapacityInteractor::new(store.clone())
            .set_capacity(Topic::new("cpu".into(), 1).unwrap())
            .is_err()
    );
    assert_eq!(
        GetTopicInteractor::new(store.clone())
            .get("cpu")
            .unwrap()
            .capacity,
        2
    );

    register(&store, "c");
    add(&store, "c", "cpu");
    add(&store, "c", "repo");

    assert!(!check(&store, "c").unwrap());
    assert_eq!(usage(&store, "repo"), 0);
    assert_eq!(task(&store, "c").state(), TaskState::Queued);

    complete(&store, "a", false);

    assert!(check(&store, "c").unwrap());
    assert_eq!(usage(&store, "cpu"), 2);
    assert_eq!(usage(&store, "repo"), 1);
}

#[test]
fn transaction_failure_rolls_back_prior_writes_and_ownership() {
    let (_dir, store) = fixture();
    topic(&store, "cpu", 1);
    register(&store, "owner");
    add(&store, "owner", "cpu");
    register(&store, "a");
    add(&store, "a", "cpu");
    store
        .connection
        .borrow()
        .execute_batch(
            "CREATE TRIGGER fail_start AFTER UPDATE OF state ON tasks
         WHEN NEW.state='running' BEGIN SELECT RAISE(ABORT,'injected'); END;",
        )
        .unwrap();

    let result = store.clone().transaction(|records| {
        records
            .topics()
            .put_topic(&Topic::new("transient".into(), 1)?)?;
        let mut task = records.tasks().task("a")?.unwrap();
        task.try_start(&[ResourceAvailability {
            topic: Topic::new("cpu".into(), 1)?,
            running: 0,
            queued_before: 0,
        }])?;
        records.tasks().save_task(&task)
    });

    assert!(result.is_err());
    assert!(
        GetTopicInteractor::new(store.clone())
            .get("transient")
            .is_err()
    );
    assert_eq!(task(&store, "a").state(), TaskState::Queued);
    assert_eq!(usage(&store, "cpu"), 1);

    store
        .connection
        .borrow()
        .execute_batch("DROP TRIGGER fail_start")
        .unwrap();
    complete(&store, "owner", false);
    assert_eq!(task(&store, "a").state(), TaskState::Running);
    store
        .connection
        .borrow()
        .execute_batch(
            "CREATE TRIGGER fail_complete AFTER UPDATE OF state ON tasks
         WHEN NEW.state='completed' BEGIN SELECT RAISE(ABORT,'injected'); END;",
        )
        .unwrap();

    assert!(
        CompleteTaskInteractor::new(store.clone())
            .complete("a", false)
            .is_err()
    );
    assert_eq!(task(&store, "a").state(), TaskState::Running);
    assert_eq!(usage(&store, "cpu"), 1);
}

#[test]
fn incompatible_databases_are_rejected_without_changing_their_bytes() {
    for version in [0, 1, 6] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("incompatible.db");
        let connection = Connection::open(&path).unwrap();
        connection
            .execute_batch(
                "CREATE TABLE unrelated(value TEXT); INSERT INTO unrelated VALUES ('preserve-me')",
            )
            .unwrap();
        connection
            .pragma_update(None, "user_version", version)
            .unwrap();
        drop(connection);
        let before = std::fs::read(&path).unwrap();

        let error = match Sqlite::open(&path) {
            Ok(_) => panic!("incompatible database opened"),
            Err(error) => error.to_string(),
        };

        assert_eq!(std::fs::read(&path).unwrap(), before);
        assert!(!path.with_extension("db-wal").exists());

        if version == 0 {
            assert!(error.contains("--db"));
        } else {
            assert!(error.contains("unsupported database schema"));
        }
    }
}

#[test]
fn autocreate_rolls_back_new_entities_after_domain_and_actual_sql_errors() {
    let (_dir, store) = fixture();
    let policy = AutoCreatePolicy {
        tasks: true,
        topics: true,
    };

    assert!(
        CheckTaskRunningInteractor::with_policy(store.clone(), policy)
            .check_running("missing")
            .is_err()
    );
    assert!(
        CompleteTaskInteractor::with_policy(store.clone(), policy)
            .complete("missing", false)
            .is_err()
    );
    assert!(
        ListTasksInteractor::new(store.clone())
            .list()
            .unwrap()
            .is_empty()
    );

    store
        .connection
        .borrow()
        .execute_batch(
            "CREATE TRIGGER fail_link BEFORE INSERT ON task_topics
         BEGIN SELECT RAISE(ABORT,'injected'); END;",
        )
        .unwrap();

    assert!(
        AddTaskToTopicInteractor::with_policy(store.clone(), policy)
            .add("job", "cpu")
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

    store
        .connection
        .borrow()
        .execute_batch("DROP TRIGGER fail_link")
        .unwrap();
    AddTaskToTopicInteractor::with_policy(store.clone(), policy)
        .add("job", "cpu")
        .unwrap();

    assert_eq!(task(&store, "job").state(), TaskState::Running);
    assert_eq!(usage(&store, "cpu"), 1);
    assert!(check(&store, "job").unwrap());
    assert!(
        AddTaskToTopicInteractor::with_policy(store.clone(), policy)
            .add("job", "new")
            .is_err()
    );
    assert_eq!(
        ListTopicsInteractor::new(store.clone())
            .list()
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn independent_connections_autocreate_one_identity_and_one_membership() {
    let (dir, store) = fixture();
    let barrier = Arc::new(Barrier::new(4));
    let children: Vec<_> = (0..4)
        .map(|_| {
            let path = dir.path().join("queue.db");
            let barrier = barrier.clone();

            thread::spawn(move || {
                let other = Sqlite::open(&path).unwrap();
                barrier.wait();
                AddTaskToTopicInteractor::with_policy(
                    other,
                    AutoCreatePolicy {
                        tasks: true,
                        topics: true,
                    },
                )
                .add("`job`", "`cpu`")
            })
        })
        .collect();

    let tasks: Vec<_> = children
        .into_iter()
        .map(|child| child.join().unwrap())
        .collect();

    assert_eq!(tasks.iter().filter(|item| item.is_ok()).count(), 1);
    assert!(
        tasks
            .iter()
            .filter_map(|item| item.as_ref().err())
            .all(|error| error.to_string().contains("already running"))
    );
    assert_eq!(
        ListTasksInteractor::new(store.clone()).list().unwrap(),
        ["job"]
    );
    assert_eq!(
        ListTopicsInteractor::new(store.clone())
            .list()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(task(&store, "job").topics(), &["cpu"]);
    assert!(check(&store, "job").unwrap());
    assert_eq!(usage(&store, "cpu"), 1);
}

fn race(path: &Path, operations: &[&str]) -> Vec<Result<bool>> {
    let barrier = Arc::new(Barrier::new(operations.len()));
    let children: Vec<_> = operations
        .iter()
        .map(|name| {
            let path = path.to_owned();
            let name = name.to_string();
            let barrier = barrier.clone();

            thread::spawn(move || {
                let store = Sqlite::open(&path).unwrap();
                barrier.wait();
                check(&store, &name)
            })
        })
        .collect();

    children
        .into_iter()
        .map(|child| child.join().unwrap())
        .collect()
}

#[test]
fn independent_connections_admit_only_one_task_and_one_waiter() {
    let (dir, store) = fixture();
    topic(&store, "cpu", 1);

    for name in ["a", "b"] {
        register(&store, name);
        add(&store, name, "cpu");
    }

    let results = race(&dir.path().join("queue.db"), &["a", "b"]);

    assert_eq!(
        results
            .iter()
            .filter(|result| matches!(result, Ok(true)))
            .count(),
        1
    );
    assert_eq!(usage(&store, "cpu"), 1);

    complete(&store, "a", false);
    let same_name = race(&dir.path().join("queue.db"), &["b", "b"]);

    assert_eq!(
        same_name
            .iter()
            .filter(|result| matches!(result, Ok(true)))
            .count(),
        2
    );
    assert_eq!(same_name.iter().filter(|result| result.is_err()).count(), 0);
    assert_eq!(usage(&store, "cpu"), 1);
}

#[test]
fn membership_racing_admission_is_included_or_rejected() {
    let (dir, store) = fixture();
    topic(&store, "cpu", 1);
    topic(&store, "repo", 1);
    register(&store, "a");
    add(&store, "a", "cpu");
    let barrier = Arc::new(Barrier::new(2));
    let path = dir.path().join("queue.db");
    let first_barrier = barrier.clone();

    let addition = thread::spawn(move || {
        let other = Sqlite::open(&path).unwrap();
        first_barrier.wait();
        AddTaskToTopicInteractor::new(other).add("a", "repo")
    });
    barrier.wait();
    let outcome = check(&store, "a").unwrap();
    let added = addition.join().unwrap();

    assert!(outcome);
    assert_eq!(usage(&store, "cpu"), 1);
    assert_eq!(usage(&store, "repo"), u64::from(added.is_ok()));
    assert_eq!(
        task(&store, "a").topics().contains(&"repo".into()),
        added.is_ok()
    );
}

#[test]
fn force_racing_admission_leaves_a_terminal_task_without_slots() {
    let (dir, store) = fixture();
    topic(&store, "cpu", 1);
    register(&store, "a");
    add(&store, "a", "cpu");
    let barrier = Arc::new(Barrier::new(2));
    let path = dir.path().join("queue.db");
    let first_barrier = barrier.clone();

    let force = thread::spawn(move || {
        let other = Sqlite::open(&path).unwrap();
        first_barrier.wait();
        CompleteTaskInteractor::new(other).complete("a", true)
    });
    barrier.wait();
    let outcome = check(&store, "a");
    force.join().unwrap().unwrap();

    assert!(outcome.is_err() || outcome == Ok(true));
    assert_eq!(task(&store, "a").state(), TaskState::Completed);
    assert_eq!(usage(&store, "cpu"), 0);
    assert!(check(&store, "a").is_err());
}

#[test]
fn independent_openers_can_initialize_a_fresh_database() {
    let dir = tempfile::tempdir().unwrap();

    for iteration in 0..12 {
        let path = dir.path().join(format!("fresh-{iteration}.db"));
        let barrier = Arc::new(Barrier::new(4));
        let threads: Vec<_> = (0..4)
            .map(|index| {
                let path = path.clone();
                let barrier = barrier.clone();

                thread::spawn(move || {
                    barrier.wait();
                    let store = Sqlite::open(&path).unwrap();
                    register(&store, &format!("task-{index}"));
                })
            })
            .collect();

        for thread in threads {
            thread.join().unwrap();
        }

        let store = Sqlite::open(&path).unwrap();

        for index in 0..4 {
            assert_eq!(
                task(&store, &format!("task-{index}")).state(),
                TaskState::Registered
            );
        }
    }
}

#[test]
fn schema_v2_upgrade_preserves_states_order_memberships_and_ownership() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("v2.db");
    let connection = Connection::open(&path).unwrap();
    connection
        .execute_batch(include_str!("schema_v2.sql"))
        .unwrap();
    drop(connection);

    let store = Sqlite::open(&path).unwrap();

    assert_eq!(task(&store, "owner").state(), TaskState::Running);
    let next = task(&store, "next");
    assert_eq!(next.state(), TaskState::Queued);
    assert_eq!(next.sequence(), 2);
    assert_eq!(next.topics(), &["cpu", "repo"]);
    assert!(next.autostart_default());
    assert!(!next.stopped());
    assert_eq!(task(&store, "done").state(), TaskState::Completed);
    assert_eq!(usage(&store, "cpu"), 1);
    assert_eq!(usage(&store, "repo"), 0);
    assert_eq!(
        GetTopicInteractor::new(store.clone())
            .get("cpu")
            .unwrap()
            .autostart,
        None
    );
    let version: i64 = store
        .connection
        .borrow()
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap();
    assert_eq!(version, 5);

    complete(&store, "owner", false);

    assert_eq!(task(&store, "next").state(), TaskState::Running);
    assert_eq!(usage(&store, "repo"), 1);
}

#[test]
fn read_only_running_check_never_admits_a_legacy_queued_task_on_a_free_resource() {
    let (_dir, store) = fixture();
    topic(&store, "cpu", 1);
    store.connection.borrow().execute_batch("INSERT INTO tasks(name,state) VALUES ('legacy','queued'); INSERT INTO task_topics VALUES (1,'cpu');").unwrap();

    assert!(!check(&store, "legacy").unwrap());
    assert!(!check(&store, "legacy").unwrap());
    assert_eq!(task(&store, "legacy").state(), TaskState::Queued);
    assert_eq!(usage(&store, "cpu"), 0);
}
mod metadata;

mod events;

mod managed;
