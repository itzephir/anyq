use super::{MemoryStore, create_topic, register};
use crate::*;
use anyq_queue_api::*;

fn metadata(key: &str, value: MetadataValue) -> Metadata {
    let mut result = Metadata::default();
    result.set(key, value).unwrap();

    result
}

fn automatic() -> QueuePolicy {
    QueuePolicy {
        autocreate: AutoCreatePolicy {
            tasks: true,
            topics: true,
        },
        ..QueuePolicy::default()
    }
}

#[test]
fn initial_metadata_is_saved_before_immediate_admission_and_creation_rolls_back() {
    let store = MemoryStore::default();
    create_topic(&store, "cpu", 1);
    create_topic(&store, "repo", 1);
    let initial = metadata("unknown.enabled", MetadataValue::Bool(true));

    let task = RegisterTaskInteractor::new(store.clone())
        .register(RegisterTaskRequest {
            name: "build".into(),
            topics: vec!["cpu".into(), "repo".into()],
            start: StartPreference::Default,
            metadata: initial.clone(),
        })
        .unwrap();

    assert_eq!(task.state(), TaskState::Running);
    assert_eq!(task.metadata(), &initial);
    assert_eq!(store.data.borrow().tasks[0].metadata(), &initial);
    let before = store.data.borrow().clone();

    assert!(
        RegisterTaskInteractor::with_options(store.clone(), automatic())
            .register(RegisterTaskRequest {
                name: "failed".into(),
                topics: vec!["created".into(), "--invalid".into()],
                start: StartPreference::Default,
                metadata: initial,
            })
            .is_err()
    );
    assert_eq!(*store.data.borrow(), before);
}

#[test]
fn focused_metadata_operations_patch_replace_and_read_without_inheritance_or_admission() {
    let store = MemoryStore::default();
    register(&store, "job");
    create_topic(&store, "cpu", 1);
    let policy = QueuePolicy::default();

    SetTaskMetadataInteractor::new(store.clone(), policy)
        .set(SetTaskMetadataRequest {
            task: "job".into(),
            key: "unknown.flag".into(),
            value: MetadataValue::Bool(true),
        })
        .unwrap();
    SetTaskMetadataInteractor::new(store.clone(), policy)
        .set(SetTaskMetadataRequest {
            task: "job".into(),
            key: "timeout".into(),
            value: MetadataValue::String("2h".into()),
        })
        .unwrap();
    SetTopicMetadataInteractor::new(store.clone(), policy)
        .set(SetTopicMetadataRequest {
            topic: "cpu".into(),
            key: "timeout".into(),
            value: MetadataValue::String("1h".into()),
        })
        .unwrap();
    let flag = GetTaskMetadataInteractor::new(store.clone(), policy)
        .get(GetTaskMetadataRequest {
            task: "job".into(),
            key: Some("unknown.flag".into()),
        })
        .unwrap();
    let timeout = GetTopicMetadataInteractor::new(store.clone(), policy)
        .get(GetTopicMetadataRequest {
            topic: "cpu".into(),
            key: Some("timeout".into()),
        })
        .unwrap();

    assert_eq!(flag, MetadataValue::Bool(true));
    assert_eq!(timeout, MetadataValue::String("1h".into()));
    assert_eq!(store.data.borrow().tasks[0].state(), TaskState::Registered);
    assert!(store.data.borrow().tasks[0].topics().is_empty());

    UnsetTaskMetadataInteractor::new(store.clone(), policy)
        .unset(UnsetTaskMetadataRequest {
            task: "job".into(),
            key: "unknown.flag".into(),
        })
        .unwrap();
    UnsetTopicMetadataInteractor::new(store.clone(), policy)
        .unset(UnsetTopicMetadataRequest {
            topic: "cpu".into(),
            key: "missing.deep".into(),
        })
        .unwrap();
    let replacement = metadata("replacement", MetadataValue::Null);
    ReplaceTaskMetadataInteractor::new(store.clone(), policy)
        .replace(ReplaceTaskMetadataRequest {
            task: "job".into(),
            metadata: replacement.clone(),
        })
        .unwrap();
    ReplaceTopicMetadataInteractor::new(store.clone(), policy)
        .replace(ReplaceTopicMetadataRequest {
            topic: "cpu".into(),
            metadata: replacement.clone(),
        })
        .unwrap();

    assert_eq!(store.data.borrow().tasks[0].metadata(), &replacement);
    assert_eq!(store.data.borrow().topics[0].metadata, replacement);
    assert!(
        store.data.borrow().tasks[0]
            .metadata()
            .get(Some("timeout"))
            .is_err()
    );
}

#[test]
fn task_metadata_freezes_at_running_and_topic_updates_preserve_running_inputs() {
    let store = MemoryStore::default();
    create_topic(&store, "cpu", 1);
    let policy = QueuePolicy::default();
    let original = metadata("timeout", MetadataValue::String("2h".into()));
    RegisterTaskInteractor::new(store.clone())
        .register(RegisterTaskRequest {
            name: "owner".into(),
            topics: vec!["cpu".into()],
            start: StartPreference::Default,
            metadata: original.clone(),
        })
        .unwrap();

    for complete in [false, true] {
        if complete {
            CompleteTaskInteractor::new(store.clone())
                .complete("owner", false)
                .unwrap();
        }
        let before = store.data.borrow().clone();

        assert!(
            SetTaskMetadataInteractor::new(store.clone(), policy)
                .set(SetTaskMetadataRequest {
                    task: "owner".into(),
                    key: "timeout".into(),
                    value: MetadataValue::Null,
                })
                .is_err()
        );
        assert!(
            UnsetTaskMetadataInteractor::new(store.clone(), policy)
                .unset(UnsetTaskMetadataRequest {
                    task: "owner".into(),
                    key: "absent".into(),
                })
                .is_err()
        );
        assert!(
            ReplaceTaskMetadataInteractor::new(store.clone(), policy)
                .replace(ReplaceTaskMetadataRequest {
                    task: "owner".into(),
                    metadata: Metadata::default(),
                })
                .is_err()
        );
        assert_eq!(*store.data.borrow(), before);

        SetTopicMetadataInteractor::new(store.clone(), policy)
            .set(SetTopicMetadataRequest {
                topic: "cpu".into(),
                key: "timeout".into(),
                value: MetadataValue::String("1h".into()),
            })
            .unwrap();

        assert_eq!(store.data.borrow().tasks[0].metadata(), &original);
    }
}

#[test]
fn failed_metadata_lookups_and_patches_roll_back_automatic_creations_and_saves() {
    let store = MemoryStore::default();
    let policy = automatic();

    assert!(
        GetTaskMetadataInteractor::new(store.clone(), policy)
            .get(GetTaskMetadataRequest {
                task: "missing".into(),
                key: Some("unknown".into()),
            })
            .is_err()
    );
    assert!(
        GetTopicMetadataInteractor::new(store.clone(), policy)
            .get(GetTopicMetadataRequest {
                topic: "missing".into(),
                key: Some("unknown".into()),
            })
            .is_err()
    );
    assert!(
        SetTaskMetadataInteractor::new(store.clone(), policy)
            .set(SetTaskMetadataRequest {
                task: "missing".into(),
                key: "a..b".into(),
                value: MetadataValue::Null,
            })
            .is_err()
    );
    assert!(
        SetTopicMetadataInteractor::new(store.clone(), policy)
            .set(SetTopicMetadataRequest {
                topic: "missing".into(),
                key: "a..b".into(),
                value: MetadataValue::Null,
            })
            .is_err()
    );
    assert!(store.data.borrow().tasks.is_empty());
    assert!(store.data.borrow().topics.is_empty());

    store.data.borrow_mut().fail_save = true;
    let before = store.data.borrow().clone();
    assert!(
        SetTaskMetadataInteractor::new(store.clone(), policy)
            .set(SetTaskMetadataRequest {
                task: "missing".into(),
                key: "valid".into(),
                value: MetadataValue::Bool(true),
            })
            .is_err()
    );
    assert_eq!(*store.data.borrow(), before);
}
