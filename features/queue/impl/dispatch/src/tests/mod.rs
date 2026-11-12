mod autostart;
mod batch;
mod memory;
mod memory_store;
mod metadata;

use crate::*;
use anyq_queue_api::*;
use memory_store::MemoryStore;

fn create_topic(store: &MemoryStore, name: &str, capacity: u32) {
    CreateTopicInteractor::new(store.clone())
        .create(Topic::new(name.into(), capacity).unwrap())
        .unwrap();
}

fn register(store: &MemoryStore, name: &str) -> Task {
    RegisterTaskInteractor::new(store.clone())
        .register(RegisterTaskRequest {
            metadata: Default::default(),
            start: anyq_queue_api::StartPreference::Default,
            name: name.into(),
            topics: Vec::new(),
        })
        .unwrap()
}

fn add(store: &MemoryStore, name: &str, topic: &str) {
    AddTaskToTopicInteractor::new(store.clone())
        .add(name, topic)
        .unwrap();
}

fn check(store: &MemoryStore, name: &str) -> Result<bool> {
    CheckTaskRunningInteractor::new(store.clone()).check_running(name)
}

#[test]
fn registration_with_initial_topics_automatically_acquires_all_resources() {
    let store = MemoryStore::default();
    create_topic(&store, "cpu", 1);
    create_topic(&store, "repo", 1);
    let request = RegisterTaskRequest {
        metadata: Default::default(),
        start: anyq_queue_api::StartPreference::Default,
        name: "build".into(),
        topics: vec!["repo".into(), "cpu".into(), "cpu".into()],
    };

    let task = RegisterTaskInteractor::new(store.clone())
        .register(request.clone())
        .unwrap();

    assert_eq!(task.state(), TaskState::Running);
    assert_eq!(task.topics(), &["cpu", "repo"]);
    assert_eq!(store.data.borrow().tasks, [task]);
    assert!(
        RegisterTaskInteractor::new(store.clone())
            .register(request)
            .is_err()
    );
    assert!(check(&store, "build").unwrap());
}

#[test]
fn failed_initial_membership_leaves_no_registered_task_or_partial_links() {
    let store = MemoryStore::default();
    create_topic(&store, "cpu", 1);
    let before = store.data.borrow().clone();

    let missing = RegisterTaskInteractor::new(store.clone()).register(RegisterTaskRequest {
        metadata: Default::default(),
        start: anyq_queue_api::StartPreference::Default,
        name: "build".into(),
        topics: vec!["cpu".into(), "missing".into()],
    });

    assert!(missing.is_err());
    assert_eq!(*store.data.borrow(), before);

    store.data.borrow_mut().fail_save = true;
    let before_save = store.data.borrow().clone();
    let failed_save = RegisterTaskInteractor::new(store.clone()).register(RegisterTaskRequest {
        metadata: Default::default(),
        start: anyq_queue_api::StartPreference::Default,
        name: "build".into(),
        topics: vec!["cpu".into()],
    });

    assert!(failed_save.is_err());
    assert_eq!(*store.data.borrow(), before_save);
}

#[test]
fn listing_and_inspection_include_the_requested_states_without_changing_them() {
    let store = MemoryStore::default();
    create_topic(&store, "cpu", 1);
    register(&store, "running");
    add(&store, "running", "cpu");
    check(&store, "running").unwrap();
    register(&store, "registered");
    register(&store, "queued");
    add(&store, "queued", "cpu");
    register(&store, "completed");
    CompleteTaskInteractor::new(store.clone())
        .complete("completed", true)
        .unwrap();
    let before = store.data.borrow().clone();

    let names = ListTasksInteractor::new(store.clone()).list().unwrap();
    let queue = GetTopicQueueInteractor::new(store.clone())
        .get("cpu")
        .unwrap();
    let queued = GetTaskInteractor::new(store.clone()).get("queued").unwrap();

    assert_eq!(names, ["completed", "queued", "registered", "running"]);
    assert_eq!(
        queue.tasks.iter().map(Task::name).collect::<Vec<_>>(),
        ["running", "queued"]
    );
    assert_eq!(queued.state(), TaskState::Queued);
    assert_eq!(queued.topics(), &["cpu"]);
    assert!(
        GetTopicQueueInteractor::new(store.clone())
            .get("missing")
            .is_err()
    );
    assert!(
        GetTaskInteractor::new(store.clone())
            .get("missing")
            .is_err()
    );
    assert_eq!(*store.data.borrow(), before);
}

#[test]
fn names_are_unique_even_after_completion_and_bad_references_have_no_side_effects() {
    let store = MemoryStore::default();
    let task = register(&store, "build");

    assert_eq!(task.sequence(), 1);
    assert!(
        RegisterTaskInteractor::new(store.clone())
            .register(RegisterTaskRequest {
                metadata: Default::default(),
                start: anyq_queue_api::StartPreference::Default,
                name: "build".into(),
                topics: Vec::new()
            })
            .is_err()
    );
    assert!(
        AddTaskToTopicInteractor::new(store.clone())
            .add("build", "missing")
            .is_err()
    );
    assert!(
        AddTaskToTopicInteractor::new(store.clone())
            .add("missing", "missing")
            .is_err()
    );
    assert_eq!(store.data.borrow().tasks[0], task);
    assert!(
        check(&store, "build")
            .unwrap_err()
            .to_string()
            .contains("not registered")
    );

    CompleteTaskInteractor::new(store.clone())
        .complete("build", true)
        .unwrap();

    assert!(
        RegisterTaskInteractor::new(store.clone())
            .register(RegisterTaskRequest {
                metadata: Default::default(),
                start: anyq_queue_api::StartPreference::Default,
                name: "build".into(),
                topics: Vec::new()
            })
            .is_err()
    );
    assert_eq!(register(&store, "next").sequence(), 2);
    assert!(
        CompleteTaskInteractor::new(store.clone())
            .complete("missing", true)
            .is_err()
    );
}

#[test]
fn crossed_membership_uses_global_registration_order_and_releases_all_resources() {
    let store = MemoryStore::default();
    create_topic(&store, "cpu", 1);
    create_topic(&store, "repo", 1);
    register(&store, "owner");
    add(&store, "owner", "cpu");
    register(&store, "a");
    register(&store, "b");
    add(&store, "a", "cpu");
    add(&store, "b", "cpu");
    add(&store, "b", "repo");
    add(&store, "a", "repo");

    assert!(!check(&store, "a").unwrap());
    assert!(!check(&store, "b").unwrap());

    CompleteTaskInteractor::new(store.clone())
        .complete("owner", false)
        .unwrap();

    assert_eq!(store.data.borrow().tasks[1].state(), TaskState::Running);
    assert_eq!(store.data.borrow().tasks[2].state(), TaskState::Queued);
    let before = store.data.borrow().clone();
    assert!(check(&store, "a").unwrap());
    assert!(check(&store, "a").unwrap());
    assert_eq!(*store.data.borrow(), before);

    CompleteTaskInteractor::new(store.clone())
        .complete("a", false)
        .unwrap();

    assert_eq!(store.data.borrow().tasks[2].state(), TaskState::Running);
    assert_eq!(store.data.borrow().tasks[2].topics(), &["cpu", "repo"]);
}

#[test]
fn an_unavailable_topic_leaves_the_whole_task_queued() {
    let store = MemoryStore::default();
    create_topic(&store, "cpu", 1);
    create_topic(&store, "repo", 1);
    register(&store, "owner");
    add(&store, "owner", "repo");
    check(&store, "owner").unwrap();
    register(&store, "build");
    add(&store, "build", "repo");
    add(&store, "build", "cpu");
    let before = store.data.borrow().clone();

    assert!(!check(&store, "build").unwrap());

    assert_eq!(*store.data.borrow(), before);
}

#[test]
fn capacity_two_allows_only_the_first_two_queued_tasks() {
    let store = MemoryStore::default();
    create_topic(&store, "cpu", 2);

    for name in ["a", "b", "c"] {
        register(&store, name);
        add(&store, name, "cpu");
    }

    assert!(!check(&store, "c").unwrap());
    assert!(check(&store, "b").unwrap());
    assert!(check(&store, "a").unwrap());
    assert!(!check(&store, "c").unwrap());
    assert!(
        SetTopicCapacityInteractor::new(store.clone())
            .set_capacity(Topic::new("cpu".into(), 1).unwrap())
            .is_err()
    );

    CompleteTaskInteractor::new(store.clone())
        .complete("a", false)
        .unwrap();

    assert!(check(&store, "c").unwrap());
    assert_eq!(store.data.borrow().topics[0].capacity, 2);
}

#[test]
fn force_removes_an_unstarted_predecessor_and_stale_wait_cannot_start_it() {
    let store = MemoryStore::default();
    create_topic(&store, "cpu", 1);
    create_topic(&store, "repo", 1);
    register(&store, "owner");
    add(&store, "owner", "repo");
    register(&store, "a");
    add(&store, "a", "repo");
    add(&store, "a", "cpu");
    register(&store, "b");
    add(&store, "b", "cpu");

    assert!(
        CompleteTaskInteractor::new(store.clone())
            .complete("a", false)
            .is_err()
    );
    assert!(!check(&store, "b").unwrap());

    CompleteTaskInteractor::new(store.clone())
        .complete("a", true)
        .unwrap();

    assert!(check(&store, "a").is_err());
    assert_eq!(store.data.borrow().tasks[2].state(), TaskState::Running);
    assert!(
        AddTaskToTopicInteractor::new(store.clone())
            .add("b", "repo")
            .is_err()
    );
}

#[test]
fn failed_saves_roll_back_admission_completion_and_membership() {
    let store = MemoryStore::default();
    create_topic(&store, "cpu", 1);
    create_topic(&store, "repo", 1);
    register(&store, "owner");
    add(&store, "owner", "cpu");
    register(&store, "build");
    add(&store, "build", "cpu");
    store.data.borrow_mut().fail_save = true;
    let before = store.data.borrow().clone();

    assert!(!check(&store, "build").unwrap());
    assert_eq!(*store.data.borrow(), before);
    assert!(
        CompleteTaskInteractor::new(store.clone())
            .complete("owner", false)
            .is_err()
    );
    assert_eq!(*store.data.borrow(), before);
    assert!(
        AddTaskToTopicInteractor::new(store.clone())
            .add("build", "repo")
            .is_err()
    );
    assert_eq!(*store.data.borrow(), before);

    store.data.borrow_mut().fail_save = false;
    store.data.borrow_mut().fail_task = Some("build".into());
    let before_promotion = store.data.borrow().clone();

    assert!(
        CompleteTaskInteractor::new(store.clone())
            .complete("owner", false)
            .is_err()
    );
    assert_eq!(*store.data.borrow(), before_promotion);
    assert!(
        SetTopicCapacityInteractor::new(store.clone())
            .set_capacity(Topic::new("cpu".into(), 2).unwrap())
            .is_err()
    );
    assert_eq!(*store.data.borrow(), before_promotion);

    store.data.borrow_mut().fail_task = None;
    CompleteTaskInteractor::new(store.clone())
        .complete("owner", false)
        .unwrap();

    assert_eq!(store.data.borrow().tasks[1].state(), TaskState::Running);
}

#[test]
fn autocreate_changes_are_staged_with_the_primary_operation_and_rollback_on_save_failure() {
    let store = MemoryStore::default();
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
    assert!(store.data.borrow().tasks.is_empty());

    store.data.borrow_mut().fail_save = true;
    let before = store.data.borrow().clone();

    assert!(
        AddTaskToTopicInteractor::with_policy(store.clone(), policy)
            .add("job", "cpu")
            .is_err()
    );
    assert_eq!(*store.data.borrow(), before);

    store.data.borrow_mut().fail_save = false;
    let task = AddTaskToTopicInteractor::with_policy(store.clone(), policy)
        .add("job", "cpu")
        .unwrap();

    assert_eq!(task.state(), TaskState::Running);
    assert_eq!(store.data.borrow().topics[0].capacity, 1);
    assert!(check(&store, "job").unwrap());

    assert!(
        AddTaskToTopicInteractor::with_policy(store.clone(), policy)
            .add("job", "new-topic")
            .is_err()
    );
    assert_eq!(store.data.borrow().topics.len(), 1);
}

mod events;
