use super::*;
use std::sync::atomic::AtomicBool;

#[test]
fn snapshot_is_current_ordered_and_observational_with_all_start_policies() {
    let (_dir, store) = fixture();
    topic(&store, "empty", 2);
    topic(&store, "cpu", 1);
    topic(&store, "repo", 1);
    let owner = RegisterTaskInteractor::new(store.clone())
        .register(RegisterTaskRequest {
            name: "owner".into(),
            topics: vec!["cpu".into(), "repo".into()],
            start: StartPreference::Default,
            metadata: Default::default(),
        })
        .unwrap();
    let stopped = RegisterTaskInteractor::new(store.clone())
        .register(RegisterTaskRequest {
            name: "stopped".into(),
            topics: vec!["cpu".into()],
            start: StartPreference::Stop,
            metadata: Default::default(),
        })
        .unwrap();
    register(&store, "unlinked");
    register(&store, "done");
    complete(&store, "done", true);

    let snapshot = ObserveQueueInteractor::new(store.clone())
        .observe(ObserveQueueRequest::default())
        .unwrap();

    assert_eq!(
        snapshot.tasks,
        [owner.clone().into(), stopped.clone().into()]
    );
    assert_eq!(snapshot.topics.len(), 3);
    assert_eq!(snapshot.count(TaskState::Running), 1);
    assert_eq!(snapshot.count(TaskState::Queued), 1);
    assert_eq!(snapshot.topic_count("repo", TaskState::Running), 1);
    assert_eq!(snapshot.topic_count("empty", TaskState::Running), 0);
    assert_eq!(task(&store, "owner"), owner);
    assert_eq!(task(&store, "stopped"), stopped);
}

#[test]
fn missing_files_and_oversized_snapshots_fail_without_replacement_or_truncation() {
    let (dir, store) = fixture();
    topic(&store, "cpu", 1);
    register(&store, "owner");
    add(&store, "owner", "cpu");
    let request = ObserveQueueRequest {
        max_entities: 1,
        ..Default::default()
    };

    let oversized = ObserveQueueInteractor::new(store.clone()).observe(request);
    let missing = dir.path().join("missing.db");
    let opened = Sqlite::open_existing(&missing, Arc::new(AtomicBool::new(false)));

    assert!(oversized.unwrap_err().to_string().contains("size limit"));
    assert!(opened.is_err());
    assert!(!missing.exists());
    assert_eq!(task(&store, "owner").state(), TaskState::Running);
}

#[test]
fn racing_completion_never_mixes_owners_across_topics() {
    let (dir, store) = fixture();
    topic(&store, "cpu", 1);
    topic(&store, "repo", 1);
    for name in ["owner", "successor"] {
        RegisterTaskInteractor::new(store.clone())
            .register(RegisterTaskRequest {
                name: name.into(),
                topics: vec!["cpu".into(), "repo".into()],
                start: StartPreference::Default,
                metadata: Default::default(),
            })
            .unwrap();
    }
    let path = dir.path().join("queue.db");
    let barrier = Arc::new(Barrier::new(2));
    let worker_barrier = barrier.clone();
    let writer = thread::spawn(move || {
        let other = Sqlite::open(&path).unwrap();
        worker_barrier.wait();

        complete(&other, "owner", false);
    });
    let mut observer = ObserveQueueInteractor::new(store);
    barrier.wait();

    for _ in 0..30 {
        let snapshot = observer.observe(ObserveQueueRequest::default()).unwrap();

        assert_eq!(snapshot.count(TaskState::Running), 1);
        assert_eq!(snapshot.topic_count("cpu", TaskState::Running), 1);
        assert_eq!(snapshot.topic_count("repo", TaskState::Running), 1);
        assert!(
            snapshot
                .tasks
                .iter()
                .filter(|task| task.state() == TaskState::Running)
                .all(|task| task.topics() == ["cpu", "repo"])
        );
    }

    writer.join().unwrap();
}
