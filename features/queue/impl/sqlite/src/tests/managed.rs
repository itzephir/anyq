use super::*;

fn running(store: &Sqlite) {
    CreateTopicInteractor::new(store.clone())
        .create(Topic::new("cpu".into(), 1).unwrap())
        .unwrap();
    for name in ["owner", "next"] {
        RegisterTaskInteractor::new(store.clone())
            .register(RegisterTaskRequest {
                name: name.into(),
                topics: vec!["cpu".into()],
                metadata: Metadata::default(),
                start: StartPreference::Default,
            })
            .unwrap();
    }
}

#[test]
fn receipt_is_atomic_with_completion_and_all_successor_events_and_replays_original_result() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("queue.db");
    let mut store = Sqlite::open(&path).unwrap();
    running(&store);
    let watermark = store.watermark().unwrap();
    let request = CompleteTaskOnceRequest {
        owner: "plugin".into(),
        action_id: "stable".into(),
        task: "owner".into(),
        force: false,
    };
    store.connection.borrow().execute_batch("CREATE TRIGGER fail_receipt BEFORE INSERT ON completion_receipts BEGIN SELECT RAISE(ABORT,'injected'); END;").unwrap();

    assert!(
        CompleteTaskOnceInteractor::new(store.clone())
            .complete(request.clone())
            .is_err()
    );

    assert_eq!(task(&store, "owner").state(), TaskState::Running);
    assert_eq!(task(&store, "next").state(), TaskState::Queued);
    assert_eq!(store.watermark().unwrap(), watermark);
    store
        .connection
        .borrow()
        .execute_batch("DROP TRIGGER fail_receipt")
        .unwrap();
    let completed = CompleteTaskOnceInteractor::new(store.clone())
        .complete(request.clone())
        .unwrap();
    let after = store.watermark().unwrap();
    drop(store);
    let mut store = Sqlite::open(&path).unwrap();
    let replay = CompleteTaskOnceInteractor::new(store.clone())
        .complete(request.clone())
        .unwrap();

    assert_eq!(replay, completed);
    assert_eq!(store.watermark().unwrap(), after);
    assert_eq!(after, watermark + 2);
    let mut conflicting = request;
    conflicting.task = "next".into();
    assert!(
        CompleteTaskOnceInteractor::new(store.clone())
            .complete(conflicting)
            .is_err()
    );
    assert_eq!(task(&store, "next").state(), TaskState::Running);
}

#[test]
fn persisted_completion_intent_prevents_late_launch_and_stopped_receipts_are_owner_scoped() {
    let directory = tempfile::tempdir().unwrap();
    let store = Sqlite::open(&directory.path().join("queue.db")).unwrap();
    running(&store);
    let request = RegisterManagedWorkRequest {
        owner: "shell".into(),
        action_id: "run".into(),
        task: "owner".into(),
    };
    let launch = RegisterManagedWorkInteractor::new(store.clone())
        .register(request.clone())
        .unwrap();

    assert!(launch.launch_allowed);
    assert!(
        !RegisterManagedWorkInteractor::new(store.clone())
            .register(request)
            .unwrap()
            .launch_allowed
    );
    assert_eq!(
        CompleteTaskInteractor::new(store.clone()).complete("owner", false),
        Err(Error::ManagedWorkPending)
    );
    assert!(
        BeginCompletionInteractor::new(store.clone())
            .begin(BeginCompletionRequest {
                task: "owner".into(),
                force: false
            })
            .unwrap()
    );
    assert!(
        RegisterManagedWorkInteractor::new(store.clone())
            .register(RegisterManagedWorkRequest {
                owner: "other".into(),
                action_id: "late".into(),
                task: "owner".into()
            })
            .is_err()
    );
    assert!(
        ReportManagedWorkInteractor::new(store.clone())
            .report(ReportManagedWorkRequest {
                owner: "other".into(),
                action_id: "run".into(),
                state: ManagedWorkState::Stopped
            })
            .is_err()
    );
    assert_eq!(usage(&store, "cpu"), 1);
    ReportManagedWorkInteractor::new(store.clone())
        .report(ReportManagedWorkRequest {
            owner: "shell".into(),
            action_id: "run".into(),
            state: ManagedWorkState::Uncertain,
        })
        .unwrap();
    assert!(
        ReportManagedWorkInteractor::new(store.clone())
            .report(ReportManagedWorkRequest {
                owner: "shell".into(),
                action_id: "run".into(),
                state: ManagedWorkState::Active
            })
            .is_err()
    );
    assert!(
        ResolveManagedWorkInteractor::new(store.clone())
            .resolve(ResolveManagedWorkRequest {
                owner: "shell".into(),
                action_id: "run".into(),
                confirmed_cleanup: false
            })
            .is_err()
    );
    ResolveManagedWorkInteractor::new(store.clone())
        .resolve(ResolveManagedWorkRequest {
            owner: "shell".into(),
            action_id: "run".into(),
            confirmed_cleanup: true,
        })
        .unwrap();
    CompleteTaskInteractor::new(store.clone())
        .complete("owner", false)
        .unwrap();

    assert_eq!(task(&store, "next").state(), TaskState::Running);
}

#[test]
fn feature_initializer_failure_rolls_back_the_entire_schema_upgrade() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("legacy.db");
    let connection = Connection::open(&path).unwrap();
    connection
        .execute_batch(include_str!("schema_v2.sql"))
        .unwrap();
    drop(connection);

    assert!(
        Sqlite::open_with_initializer(
            &path,
            std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            |connection| {
                connection
                    .execute_batch("CREATE TABLE feature_partial(value TEXT)")
                    .unwrap();
                Err(Error::Storage("injected feature migration failure".into()))
            }
        )
        .is_err()
    );

    let connection = Connection::open(&path).unwrap();
    let version: i64 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap();
    let tables:i64=connection.query_row("SELECT count(*) FROM sqlite_schema WHERE name IN ('queue_events','managed_work','feature_partial')",[],|row|row.get(0)).unwrap();
    assert_eq!(version, 2);
    assert_eq!(tables, 0);
}

#[test]
fn independent_launch_and_completion_transactions_serialize_the_same_task() {
    use std::sync::{Arc, Barrier};

    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("race.db");
    let store = Sqlite::open(&path).unwrap();
    running(&store);
    let barrier = Arc::new(Barrier::new(2));
    let launch_path = path.clone();
    let launch_barrier = barrier.clone();
    let launch = std::thread::spawn(move || {
        let store = Sqlite::open(&launch_path).unwrap();
        launch_barrier.wait();
        RegisterManagedWorkInteractor::new(store).register(RegisterManagedWorkRequest {
            owner: "worker".into(),
            action_id: "race".into(),
            task: "owner".into(),
        })
    });
    let completion = std::thread::spawn(move || {
        let store = Sqlite::open(&path).unwrap();
        barrier.wait();
        BeginCompletionInteractor::new(store)
            .begin(BeginCompletionRequest {
                task: "owner".into(),
                force: false,
            })
            .unwrap()
    });

    let launched = launch.join().unwrap().is_ok();
    let blocked = completion.join().unwrap();

    assert_eq!(blocked, launched);
    if launched {
        assert_eq!(
            CompleteTaskInteractor::new(store.clone()).complete("owner", false),
            Err(Error::ManagedWorkPending)
        );
        ReportManagedWorkInteractor::new(store.clone())
            .report(ReportManagedWorkRequest {
                owner: "worker".into(),
                action_id: "race".into(),
                state: ManagedWorkState::Stopped,
            })
            .unwrap();
    }
    CompleteTaskInteractor::new(store.clone())
        .complete("owner", false)
        .unwrap();
    assert_eq!(task(&store, "next").state(), TaskState::Running);
}
