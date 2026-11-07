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
        .register(name.into())
        .unwrap();
}

fn add(store: &Sqlite, name: &str, resource: &str) {
    AddTaskToTopicInteractor::new(store.clone())
        .add(name, resource)
        .unwrap();
}

fn start(store: &Sqlite, name: &str) -> Result<StartOutcome> {
    TryStartTaskInteractor::new(store.clone()).try_start(name)
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
                topic: resource.into(),
                state: TaskState::Running,
                before_sequence: None,
            })
        })
        .unwrap()
}

#[test]
fn every_lifecycle_state_and_global_order_survive_reopening() {
    let (dir, store) = fixture();
    topic(&store, "cpu", 1);
    topic(&store, "repo", 1);
    register(&store, "done");
    add(&store, "done", "cpu");
    add(&store, "done", "repo");
    start(&store, "done").unwrap();
    complete(&store, "done", false);
    register(&store, "registered");
    register(&store, "queued");
    add(&store, "queued", "repo");
    register(&store, "running");
    add(&store, "running", "cpu");
    start(&store, "running").unwrap();
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
            .register("done".into())
            .is_err()
    );
}

#[test]
fn sqlite_counts_all_memberships_and_obeys_global_order() {
    let (_dir, store) = fixture();
    topic(&store, "cpu", 1);
    topic(&store, "repo", 1);
    register(&store, "a");
    register(&store, "b");
    add(&store, "a", "cpu");
    add(&store, "b", "repo");
    add(&store, "a", "repo");
    add(&store, "b", "cpu");
    add(&store, "a", "cpu");

    assert_eq!(start(&store, "b").unwrap(), StartOutcome::Busy);
    assert_eq!(usage(&store, "cpu"), 0);
    assert_eq!(usage(&store, "repo"), 0);
    assert_eq!(start(&store, "a").unwrap(), StartOutcome::Started);
    assert_eq!(usage(&store, "cpu"), 1);
    assert_eq!(usage(&store, "repo"), 1);

    complete(&store, "a", false);

    assert_eq!(start(&store, "b").unwrap(), StartOutcome::Started);
    assert_eq!(task(&store, "b").topics(), &["cpu", "repo"]);
    assert!(start(&store, "b").is_err());
}

#[test]
fn capacity_reduction_and_partial_admission_are_rejected() {
    let (_dir, store) = fixture();
    topic(&store, "cpu", 2);
    topic(&store, "repo", 1);

    for name in ["a", "b"] {
        register(&store, name);
        add(&store, name, "cpu");
        start(&store, name).unwrap();
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

    assert_eq!(start(&store, "c").unwrap(), StartOutcome::Busy);
    assert_eq!(usage(&store, "repo"), 0);
    assert_eq!(task(&store, "c").state(), TaskState::Queued);

    complete(&store, "a", false);

    assert_eq!(start(&store, "c").unwrap(), StartOutcome::Started);
    assert_eq!(usage(&store, "cpu"), 2);
    assert_eq!(usage(&store, "repo"), 1);
}

#[test]
fn transaction_failure_rolls_back_prior_writes_and_ownership() {
    let (_dir, store) = fixture();
    topic(&store, "cpu", 1);
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
    assert_eq!(usage(&store, "cpu"), 0);

    store
        .connection
        .borrow()
        .execute_batch("DROP TRIGGER fail_start")
        .unwrap();
    start(&store, "a").unwrap();
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
    for version in [0, 1, 3] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("legacy.db");
        let connection = Connection::open(&path).unwrap();
        connection
            .execute_batch(
                "CREATE TABLE messages(payload TEXT); INSERT INTO messages VALUES ('preserve-me')",
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

        if version <= 1 {
            assert!(error.contains("--db"));
        } else {
            assert!(error.contains("unsupported database schema"));
        }
    }
}

fn race(path: &Path, operations: &[&str]) -> Vec<Result<StartOutcome>> {
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
                start(&store, &name)
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
            .filter(|result| matches!(result, Ok(StartOutcome::Started)))
            .count(),
        1
    );
    assert_eq!(usage(&store, "cpu"), 1);

    complete(&store, "a", false);
    let same_name = race(&dir.path().join("queue.db"), &["b", "b"]);

    assert_eq!(
        same_name
            .iter()
            .filter(|result| matches!(result, Ok(StartOutcome::Started)))
            .count(),
        1
    );
    assert_eq!(same_name.iter().filter(|result| result.is_err()).count(), 1);
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
    let outcome = start(&store, "a").unwrap();
    let added = addition.join().unwrap();

    assert_eq!(outcome, StartOutcome::Started);
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
    let outcome = start(&store, "a");
    force.join().unwrap().unwrap();

    assert!(outcome.is_err() || outcome == Ok(StartOutcome::Started));
    assert_eq!(task(&store, "a").state(), TaskState::Completed);
    assert_eq!(usage(&store, "cpu"), 0);
    assert!(start(&store, "a").is_err());
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
