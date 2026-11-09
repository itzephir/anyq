use super::*;

#[test]
fn stopped_tasks_never_autostart_and_failed_manual_start_does_not_change_them() {
    let store = MemoryStore::default();
    create_topic(&store, "cpu", 1);
    register(&store, "owner");
    add(&store, "owner", "cpu");
    let stopped = RegisterTaskInteractor::new(store.clone())
        .register(RegisterTaskRequest {
            name: "manual".into(),
            topics: vec!["cpu".into()],
            start: StartPreference::Stop,
        })
        .unwrap();
    let before = store.data.borrow().clone();

    let outcome = StartTaskInteractor::new(store.clone())
        .start("manual")
        .unwrap();

    assert_eq!(outcome, StartTaskOutcome::Busy);
    assert_eq!(*store.data.borrow(), before);
    assert!(stopped.stopped());
    CompleteTaskInteractor::new(store.clone())
        .complete("owner", false)
        .unwrap();
    assert_eq!(store.data.borrow().tasks[1].state(), TaskState::Queued);

    assert_eq!(
        StartTaskInteractor::new(store.clone())
            .start("manual")
            .unwrap(),
        StartTaskOutcome::Started
    );
    assert_eq!(
        StartTaskInteractor::new(store.clone())
            .start("manual")
            .unwrap(),
        StartTaskOutcome::AlreadyRunning
    );
    assert_eq!(store.data.borrow().tasks[1].state(), TaskState::Running);
}

#[test]
fn failed_explicit_creation_start_rolls_back_registration_and_autocreate() {
    let store = MemoryStore::default();
    create_topic(&store, "cpu", 1);
    register(&store, "owner");
    add(&store, "owner", "cpu");
    let policy = QueuePolicy {
        autocreate: AutoCreatePolicy {
            tasks: true,
            topics: true,
        },
        task_autostart: false,
    };
    let before = store.data.borrow().clone();

    let outcome =
        RegisterTaskInteractor::with_options(store.clone(), policy).register(RegisterTaskRequest {
            name: "manual".into(),
            topics: vec!["cpu".into(), "transient".into()],
            start: StartPreference::Start,
        });

    assert!(outcome.is_err());
    assert_eq!(*store.data.borrow(), before);
}

#[test]
fn enabled_autostart_makes_explicit_creation_start_a_noop_when_resources_are_busy() {
    let store = MemoryStore::default();
    create_topic(&store, "cpu", 1);
    register(&store, "owner");
    add(&store, "owner", "cpu");

    let queued = RegisterTaskInteractor::new(store.clone())
        .register(RegisterTaskRequest {
            name: "next".into(),
            topics: vec!["cpu".into()],
            start: StartPreference::Start,
        })
        .unwrap();

    assert_eq!(queued.state(), TaskState::Queued);
    CompleteTaskInteractor::new(store.clone())
        .complete("owner", false)
        .unwrap();
    assert_eq!(store.data.borrow().tasks[1].state(), TaskState::Running);
}

#[test]
fn topic_enablement_and_its_promotions_roll_back_together_on_save_failure() {
    let store = MemoryStore::default();
    create_topic(&store, "cpu", 1);
    SetTopicAutostartInteractor::new(store.clone())
        .set_autostart("cpu", false)
        .unwrap();
    register(&store, "next");
    add(&store, "next", "cpu");
    store.data.borrow_mut().fail_task = Some("next".into());
    let before = store.data.borrow().clone();

    let outcome = SetTopicAutostartInteractor::new(store.clone()).set_autostart("cpu", true);

    assert!(outcome.is_err());
    assert_eq!(*store.data.borrow(), before);
}
