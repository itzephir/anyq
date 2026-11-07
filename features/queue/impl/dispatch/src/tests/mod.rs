mod memory;
mod memory_store;

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
        .register(name.into())
        .unwrap()
}

fn add(store: &MemoryStore, name: &str, topic: &str) {
    AddTaskToTopicInteractor::new(store.clone())
        .add(name, topic)
        .unwrap();
}

fn start(store: &MemoryStore, name: &str) -> Result<StartOutcome> {
    TryStartTaskInteractor::new(store.clone()).try_start(name)
}

#[test]
fn names_are_unique_even_after_completion_and_bad_references_have_no_side_effects() {
    let store = MemoryStore::default();
    let task = register(&store, "build");

    assert_eq!(task.sequence(), 1);
    assert!(
        RegisterTaskInteractor::new(store.clone())
            .register("build".into())
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
        start(&store, "build")
            .unwrap_err()
            .to_string()
            .contains("not registered")
    );

    CompleteTaskInteractor::new(store.clone())
        .complete("build", true)
        .unwrap();

    assert!(
        RegisterTaskInteractor::new(store.clone())
            .register("build".into())
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
    register(&store, "a");
    register(&store, "b");
    add(&store, "a", "cpu");
    add(&store, "b", "repo");
    add(&store, "a", "repo");
    add(&store, "b", "cpu");

    assert_eq!(start(&store, "b").unwrap(), StartOutcome::Busy);
    assert_eq!(start(&store, "a").unwrap(), StartOutcome::Started);
    assert_eq!(start(&store, "b").unwrap(), StartOutcome::Busy);
    assert!(start(&store, "a").is_err());

    CompleteTaskInteractor::new(store.clone())
        .complete("a", false)
        .unwrap();

    assert_eq!(start(&store, "b").unwrap(), StartOutcome::Started);
    assert_eq!(store.data.borrow().tasks[1].topics(), &["cpu", "repo"]);
}

#[test]
fn an_unavailable_topic_leaves_the_whole_task_queued() {
    let store = MemoryStore::default();
    create_topic(&store, "cpu", 1);
    create_topic(&store, "repo", 1);
    register(&store, "owner");
    add(&store, "owner", "repo");
    start(&store, "owner").unwrap();
    register(&store, "build");
    add(&store, "build", "cpu");
    add(&store, "build", "repo");
    let before = store.data.borrow().clone();

    assert_eq!(start(&store, "build").unwrap(), StartOutcome::Busy);

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

    assert_eq!(start(&store, "c").unwrap(), StartOutcome::Busy);
    assert_eq!(start(&store, "b").unwrap(), StartOutcome::Started);
    assert_eq!(start(&store, "a").unwrap(), StartOutcome::Started);
    assert_eq!(start(&store, "c").unwrap(), StartOutcome::Busy);
    assert!(
        SetTopicCapacityInteractor::new(store.clone())
            .set_capacity(Topic::new("cpu".into(), 1).unwrap())
            .is_err()
    );

    CompleteTaskInteractor::new(store.clone())
        .complete("a", false)
        .unwrap();

    assert_eq!(start(&store, "c").unwrap(), StartOutcome::Started);
    assert_eq!(store.data.borrow().topics[0].capacity, 2);
}

#[test]
fn force_removes_an_unstarted_predecessor_and_stale_wait_cannot_start_it() {
    let store = MemoryStore::default();
    create_topic(&store, "cpu", 1);
    register(&store, "a");
    register(&store, "b");
    add(&store, "a", "cpu");
    add(&store, "b", "cpu");

    assert!(
        CompleteTaskInteractor::new(store.clone())
            .complete("a", false)
            .is_err()
    );
    assert_eq!(start(&store, "b").unwrap(), StartOutcome::Busy);

    CompleteTaskInteractor::new(store.clone())
        .complete("a", true)
        .unwrap();

    assert!(start(&store, "a").is_err());
    assert_eq!(start(&store, "b").unwrap(), StartOutcome::Started);
    assert!(
        AddTaskToTopicInteractor::new(store.clone())
            .add("b", "cpu")
            .is_err()
    );
}

#[test]
fn failed_saves_roll_back_admission_completion_and_membership() {
    let store = MemoryStore::default();
    create_topic(&store, "cpu", 1);
    register(&store, "build");
    add(&store, "build", "cpu");
    store.data.borrow_mut().fail_save = true;
    let before = store.data.borrow().clone();

    assert!(start(&store, "build").is_err());
    assert_eq!(*store.data.borrow(), before);
    assert!(
        CompleteTaskInteractor::new(store.clone())
            .complete("build", true)
            .is_err()
    );
    assert_eq!(*store.data.borrow(), before);

    create_topic(&store, "repo", 1);
    let before_add = store.data.borrow().clone();

    assert!(
        AddTaskToTopicInteractor::new(store.clone())
            .add("build", "repo")
            .is_err()
    );
    assert_eq!(*store.data.borrow(), before_add);

    store.data.borrow_mut().fail_save = false;
    assert_eq!(start(&store, "build").unwrap(), StartOutcome::Started);
    store.data.borrow_mut().fail_save = true;
    let running = store.data.borrow().clone();

    assert!(
        CompleteTaskInteractor::new(store.clone())
            .complete("build", false)
            .is_err()
    );
    assert_eq!(*store.data.borrow(), running);
}
