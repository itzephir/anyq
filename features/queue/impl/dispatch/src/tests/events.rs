use super::*;

#[test]
fn real_transitions_have_ordered_snapshots_and_noops_have_none() {
    let store = MemoryStore::default();
    create_topic(&store, "cpu", 1);
    let mut request = RegisterTaskRequest {
        name: "job".into(),
        topics: vec!["cpu".into(), "cpu".into()],
        metadata: Metadata::default(),
        start: StartPreference::Default,
    };
    request
        .metadata
        .set("timeout", MetadataValue::String("2h".into()))
        .unwrap();

    RegisterTaskInteractor::new(store.clone())
        .register(request)
        .unwrap();
    let events = store.data.borrow().events.clone();

    assert_eq!(
        events.iter().map(|event| event.kind).collect::<Vec<_>>(),
        [
            QueueEventKind::TopicCreated,
            QueueEventKind::TaskRegistered,
            QueueEventKind::TaskLinked,
            QueueEventKind::TaskRunning,
        ]
    );
    let running = events.last().unwrap();
    assert_eq!(running.occurred_at, 1234);
    assert_eq!(
        running
            .task
            .as_ref()
            .unwrap()
            .metadata()
            .get(Some("timeout"))
            .unwrap(),
        &MetadataValue::String("2h".into())
    );
    assert_eq!(running.topics.len(), 1);

    StartTaskInteractor::new(store.clone())
        .start("job")
        .unwrap();
    create_topic(&store, "cpu", 1);
    SetTopicCapacityInteractor::new(store.clone())
        .set_capacity(Topic::new("cpu".into(), 1).unwrap())
        .unwrap();

    assert_eq!(store.data.borrow().events, events);
    SetTopicAutostartInteractor::new(store.clone())
        .set_autostart("cpu", false)
        .unwrap();
    let before = store.data.borrow().events.len();
    SetTopicAutostartInteractor::new(store.clone())
        .set_autostart("cpu", false)
        .unwrap();

    assert_eq!(store.data.borrow().events.len(), before);
    assert_eq!(running.topics[0].autostart, None);
}

#[test]
fn completion_and_all_successor_events_roll_back_after_a_late_save_failure() {
    let store = MemoryStore::default();
    create_topic(&store, "cpu", 1);
    create_topic(&store, "repo", 1);
    for (name, topics) in [
        ("owner", vec!["cpu", "repo"]),
        ("first", vec!["cpu"]),
        ("second", vec!["repo"]),
    ] {
        RegisterTaskInteractor::new(store.clone())
            .register(RegisterTaskRequest {
                name: name.into(),
                topics: topics.into_iter().map(Into::into).collect(),
                metadata: Metadata::default(),
                start: StartPreference::Default,
            })
            .unwrap();
    }
    store.data.borrow_mut().fail_task = Some("second".into());
    let before = store.data.borrow().clone();

    assert!(
        CompleteTaskInteractor::new(store.clone())
            .complete("owner", false)
            .is_err()
    );

    assert_eq!(*store.data.borrow(), before);
    store.data.borrow_mut().fail_task = None;
    CompleteTaskInteractor::new(store.clone())
        .complete("owner", false)
        .unwrap();
    let data = store.data.borrow();
    let tail = &data.events[before.events.len()..];

    assert_eq!(
        tail.iter().map(|event| event.kind).collect::<Vec<_>>(),
        [
            QueueEventKind::TaskCompleted,
            QueueEventKind::TaskRunning,
            QueueEventKind::TaskRunning
        ]
    );
    assert_eq!(
        tail.iter()
            .map(|event| event.task.as_ref().unwrap().name())
            .collect::<Vec<_>>(),
        ["owner", "first", "second"]
    );
}

#[test]
fn metadata_noops_and_failed_creation_do_not_publish_events() {
    let store = MemoryStore::default();
    register(&store, "job");
    let before = store.data.borrow().clone();

    UnsetTaskMetadataInteractor::new(store.clone(), QueuePolicy::default())
        .unset(UnsetTaskMetadataRequest {
            task: "job".into(),
            key: "missing".into(),
        })
        .unwrap();
    ReplaceTaskMetadataInteractor::new(store.clone(), QueuePolicy::default())
        .replace(ReplaceTaskMetadataRequest {
            task: "job".into(),
            metadata: Metadata::default(),
        })
        .unwrap();
    assert!(
        RegisterTaskInteractor::new(store.clone())
            .register(RegisterTaskRequest {
                name: "bad".into(),
                topics: vec!["missing".into()],
                metadata: Metadata::default(),
                start: StartPreference::Default
            })
            .is_err()
    );

    assert_eq!(*store.data.borrow(), before);
}
