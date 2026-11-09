use super::*;

#[test]
fn batch_topics_are_attached_before_admission_and_errors_roll_back_every_link() {
    let store = MemoryStore::default();
    create_topic(&store, "cpu", 1);
    create_topic(&store, "repo", 1);
    register(&store, "build");
    let before = store.data.borrow().clone();

    let failed = AddTaskToTopicsInteractor::new(store.clone()).add(AddTaskToTopicsRequest {
        task: "build".into(),
        topics: vec!["cpu".into(), "missing".into()],
    });

    assert!(failed.is_err());
    assert_eq!(*store.data.borrow(), before);

    let task = AddTaskToTopicsInteractor::new(store.clone())
        .add(AddTaskToTopicsRequest {
            task: "build".into(),
            topics: vec!["repo".into(), "cpu".into(), "cpu".into()],
        })
        .unwrap();

    assert_eq!(task.state(), TaskState::Running);
    assert_eq!(task.topics(), &["cpu", "repo"]);
}

#[test]
fn batch_tasks_follow_registration_order_instead_of_argument_order() {
    let store = MemoryStore::default();
    create_topic(&store, "cpu", 1);

    for name in ["z-first", "a-second", "m-third"] {
        register(&store, name);
    }

    let tasks = AddTasksToTopicInteractor::new(store.clone())
        .add(AddTasksToTopicRequest {
            tasks: vec![
                "m-third".into(),
                "a-second".into(),
                "z-first".into(),
                "z-first".into(),
            ],
            topic: "cpu".into(),
        })
        .unwrap();

    assert_eq!(
        tasks.iter().map(Task::name).collect::<Vec<_>>(),
        ["z-first", "a-second", "m-third"]
    );
    assert_eq!(
        tasks.iter().map(Task::state).collect::<Vec<_>>(),
        [TaskState::Running, TaskState::Queued, TaskState::Queued]
    );
}

#[test]
fn batch_errors_roll_back_prior_tasks_and_automatic_creations() {
    let store = MemoryStore::default();
    let policy = AutoCreatePolicy {
        tasks: true,
        topics: true,
    };
    register(&store, "completed");
    CompleteTaskInteractor::new(store.clone())
        .complete("completed", true)
        .unwrap();
    let before = store.data.borrow().clone();

    let result =
        AddTasksToTopicInteractor::with_policy(store.clone(), policy).add(AddTasksToTopicRequest {
            tasks: vec!["new-task".into(), "completed".into()],
            topic: "new-topic".into(),
        });

    assert!(result.is_err());
    assert_eq!(*store.data.borrow(), before);

    let result =
        AddTaskToTopicsInteractor::with_policy(store.clone(), policy).add(AddTaskToTopicsRequest {
            task: "new-task".into(),
            topics: vec!["new-topic".into(), "1-invalid".into()],
        });

    assert!(result.is_err());
    assert_eq!(*store.data.borrow(), before);
}

#[test]
fn increased_capacity_fills_multiple_slots_without_a_waiter() {
    let store = MemoryStore::default();
    create_topic(&store, "cpu", 1);

    for name in ["a", "b", "c", "d"] {
        register(&store, name);
        add(&store, name, "cpu");
    }

    SetTopicCapacityInteractor::new(store.clone())
        .set_capacity(Topic::new("cpu".into(), 3).unwrap())
        .unwrap();

    assert_eq!(
        store
            .data
            .borrow()
            .tasks
            .iter()
            .map(Task::state)
            .collect::<Vec<_>>(),
        [
            TaskState::Running,
            TaskState::Running,
            TaskState::Running,
            TaskState::Queued
        ]
    );
}

#[test]
fn unrelated_resources_can_run_while_an_earlier_task_is_blocked() {
    let store = MemoryStore::default();
    create_topic(&store, "cpu", 1);
    create_topic(&store, "repo", 1);
    register(&store, "owner");
    add(&store, "owner", "cpu");
    register(&store, "blocked");
    add(&store, "blocked", "cpu");
    register(&store, "independent");

    add(&store, "independent", "repo");

    assert_eq!(store.data.borrow().tasks[1].state(), TaskState::Queued);
    assert_eq!(store.data.borrow().tasks[2].state(), TaskState::Running);
}
