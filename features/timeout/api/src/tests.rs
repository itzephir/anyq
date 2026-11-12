use super::*;
use anyq_queue_api::*;

fn event() -> QueueEvent {
    let task = Task::restore(1, "job".into(), TaskState::Running, vec!["cpu".into()]).unwrap();
    QueueEvent {
        database_id: "db".into(),
        sequence: 9,
        version: 1,
        snapshot: EventSnapshot {
            kind: QueueEventKind::TaskRunning,
            occurred_at: 123,
            task: Some(task),
            topics: vec![Topic::new("cpu".into(), 1).unwrap()],
            directory: None,
        },
    }
}

#[test]
fn durations_are_positive_checked_and_unambiguous() {
    for (text, expected) in [
        ("1ms", 1),
        ("2s", 2000),
        ("3m", 180000),
        ("2h", 7200000),
        ("1d", 86400000),
    ] {
        assert_eq!(
            Duration::parse(text).unwrap().deadline(0).unwrap(),
            expected
        );
    }
    for invalid in [
        "",
        "0ms",
        "-2h",
        "+2h",
        "1.5h",
        "2",
        " 2h",
        "2h ",
        "1H",
        "9223372036854775807d",
        "999999999999999999999999s",
    ] {
        assert!(Duration::parse(invalid).is_err(), "{invalid}");
    }
    assert!(Duration::parse("1ms").unwrap().deadline(i64::MAX).is_err());
}

#[test]
fn shortest_snapshot_wins_and_saved_identity_never_moves() {
    let mut event = event();
    assert_eq!(resolve(&event).unwrap(), None);
    let mut metadata = Metadata::default();
    metadata
        .set("timeout", MetadataValue::String("2h".into()))
        .unwrap();
    let task = event.snapshot.task.take().unwrap();
    event.snapshot.task = Some(
        Task::restore_with_metadata(
            1,
            task.name().into(),
            TaskState::Running,
            task.topics().to_vec(),
            metadata,
        )
        .unwrap(),
    );
    event.snapshot.topics[0]
        .metadata
        .set("timeout", MetadataValue::String("30m".into()))
        .unwrap();

    let plan = resolve(&event).unwrap().unwrap();

    assert_eq!(plan.deadline, 123 + 1_800_000);
    assert_eq!(plan.source, "topic:cpu");
    assert_eq!(plan.event_id, "db:9");
    assert_eq!(Deadline::restore(plan.metadata().unwrap()).unwrap(), plan);
    assert_eq!(resolve(&event).unwrap().unwrap(), plan);
}

#[test]
fn invalid_source_is_preserved_without_selecting_another_valid_duration() {
    let mut event = event();
    event.snapshot.topics[0]
        .metadata
        .set("timeout", MetadataValue::Bool(false))
        .unwrap();

    let error = resolve(&event).unwrap_err().to_string();

    assert!(error.contains("topic:cpu"));
    assert!(error.contains("Bool(false)"));
}
