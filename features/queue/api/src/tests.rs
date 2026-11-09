use crate::*;

#[test]
fn unquoted_commands_and_non_latin_initials_require_the_explicit_wrapper() {
    for name in [
        "list", "create", "capacity", "link", "add", "wait", "complete", "config", "help", "-",
        "--", "-cpu", "--list", "9cpu", "🔥",
    ] {
        let mut task = Task::new(1, "build".into()).unwrap();

        assert!(Topic::new(name.into(), 1).is_err(), "{name}");
        assert!(Task::new(1, name.into()).is_err(), "{name}");
        assert!(task.add_topic(name).is_err(), "{name}");
        assert_eq!(Topic::new(format!("`{name}`"), 1).unwrap().name, name);
        assert_eq!(Task::new(1, format!("`{name}`")).unwrap().name(), name);
        assert!(Topic::restore(name.into(), 1).is_ok());
        assert_eq!(task.state(), TaskState::Registered);
        assert!(task.topics().is_empty());
    }

    for name in ["cpu", "large-builds", "task", "topic", "Create"] {
        assert!(Topic::new(name.into(), 1).is_ok(), "{name}");
    }
}

#[test]
fn rich_names_and_literal_backtick_wrappers_have_one_canonical_identity() {
    for (input, value) in [
        (
            "a spaces/символы #$%@!^&*+~?><,.;:(){}[]\"'",
            "a spaces/символы #$%@!^&*+~?><,.;:(){}[]\"'",
        ),
        ("`build`", "build"),
        ("`🔥 сборка`", "🔥 сборка"),
        ("`--cpu`", "--cpu"),
        ("``literal``", "`literal`"),
        ("``", ""),
        ("`🔥\n\0`", "🔥\n\0"),
    ] {
        let mut task = Task::new(1, input.into()).unwrap();
        let topic = Topic::new(input.into(), 1).unwrap();

        task.add_topic(input).unwrap();
        let restored =
            Task::restore(1, task.name().into(), task.state(), task.topics().to_vec()).unwrap();

        assert_eq!(task.name(), value);
        assert_eq!(topic.name, value);
        assert_eq!(task.topics(), &[value]);
        assert_eq!(restored, task);
        assert_eq!(parse_name(&format_name(value)).unwrap(), value);
    }

    assert!(Task::new(1, "a".repeat(1024)).is_ok());
}

#[test]
fn bare_names_require_an_ascii_latin_initial_and_balanced_outer_backticks() {
    for input in [
        "",
        "9task",
        "-task",
        "--topic",
        "🔥",
        "задача",
        "`unclosed",
        "'",
    ] {
        assert!(Task::new(1, input.into()).is_err(), "{input}");
        assert!(Topic::new(input.into(), 1).is_err(), "{input}");
    }
}

#[test]
fn current_topic_queue_defines_active_states_and_orders_by_registration() {
    let topic = Topic::new("cpu".into(), 1).unwrap();
    let later = Task::restore(2, "a-later".into(), TaskState::Queued, vec!["cpu".into()]).unwrap();
    let earlier = Task::restore(
        1,
        "z-earlier".into(),
        TaskState::Running,
        vec!["cpu".into()],
    )
    .unwrap();

    let queue = TopicQueue::new(topic, vec![later, earlier]);

    assert_eq!(TopicQueue::STATES, [TaskState::Queued, TaskState::Running]);
    assert_eq!(
        queue.tasks.iter().map(Task::name).collect::<Vec<_>>(),
        ["z-earlier", "a-later"]
    );
}

fn resource(name: &str, capacity: u32, running: u64, queued_before: u64) -> ResourceAvailability {
    ResourceAvailability {
        topic: Topic::new(name.into(), capacity).unwrap(),
        running,
        queued_before,
    }
}

#[test]
fn registration_membership_start_and_completion_follow_one_lifecycle() {
    let mut task = Task::new(1, "build".into()).unwrap();

    assert_eq!(task.state(), TaskState::Registered);
    assert!(
        task.require_queued()
            .unwrap_err()
            .to_string()
            .contains("not registered in any topic")
    );
    assert!(
        task.complete(false)
            .unwrap_err()
            .to_string()
            .contains("was not run")
    );

    task.add_topic("repo").unwrap();
    task.add_topic("cpu").unwrap();
    task.add_topic("cpu").unwrap();

    assert_eq!(task.topics(), &["cpu", "repo"]);
    assert_eq!(task.state(), TaskState::Queued);
    assert!(task.complete(false).is_err());

    let outcome = task
        .try_start(&[resource("repo", 1, 0, 0), resource("cpu", 1, 0, 0)])
        .unwrap();

    assert_eq!(outcome, StartOutcome::Started);
    assert_eq!(task.state(), TaskState::Running);
    assert!(task.add_topic("another").is_err());
    assert!(task.add_topic("cpu").is_err());
    assert!(
        task.try_start(&[resource("repo", 1, 0, 0), resource("cpu", 1, 0, 0)])
            .is_err()
    );

    task.complete(false).unwrap();

    assert_eq!(task.state(), TaskState::Completed);
    assert_eq!(task.topics(), &["cpu", "repo"]);
    assert!(task.complete(false).is_err());
    assert!(task.complete(true).is_err());
    assert!(task.add_topic("cpu").is_err());
    assert!(task.require_queued().is_err());
}

#[test]
fn unavailable_resource_never_partially_starts_a_task() {
    let mut task = Task::new(1, "build".into()).unwrap();
    task.add_topic("cpu").unwrap();
    task.add_topic("repo").unwrap();
    let before = task.clone();

    let outcome = task
        .try_start(&[resource("cpu", 1, 0, 0), resource("repo", 1, 1, 0)])
        .unwrap();

    assert_eq!(outcome, StartOutcome::Busy);
    assert_eq!(task, before);
}

#[test]
fn admission_checks_capacity_and_global_predecessors_without_overflow() {
    for (capacity, running, earlier, expected) in [
        (1, 0, 0, true),
        (1, 1, 0, false),
        (1, 0, 1, false),
        (2, 0, 1, true),
        (2, 1, 1, false),
        (2, 1, 0, true),
        (2, u64::MAX, 0, false),
        (2, 0, u64::MAX, false),
    ] {
        assert_eq!(
            resource("cpu", capacity, running, earlier).permits_start(),
            expected
        );
    }
}

#[test]
fn every_topic_must_have_exactly_one_matching_snapshot() {
    let mut task = Task::new(1, "build".into()).unwrap();
    task.add_topic("cpu").unwrap();
    task.add_topic("repo").unwrap();
    let before = task.clone();

    for resources in [
        vec![resource("cpu", 1, 0, 0)],
        vec![resource("cpu", 1, 0, 0), resource("cpu", 1, 0, 0)],
        vec![resource("cpu", 1, 0, 0), resource("other", 1, 0, 0)],
    ] {
        assert!(task.try_start(&resources).is_err());
        assert_eq!(task, before);
    }
}

#[test]
fn force_completes_registered_queued_and_running_tasks() {
    for initial in [TaskState::Registered, TaskState::Queued, TaskState::Running] {
        let topics = if initial == TaskState::Registered {
            vec![]
        } else {
            vec!["cpu".into()]
        };
        let mut task = Task::restore(1, "build".into(), initial, topics).unwrap();

        task.complete(true).unwrap();

        assert_eq!(task.state(), TaskState::Completed);
        assert!(task.require_queued().is_err());
    }
}

#[test]
fn invalid_names_sequences_and_persisted_membership_are_rejected() {
    assert!(Task::new(0, "build".into()).is_err());
    assert!(Task::new(1, "9bad name".into()).is_err());
    assert!(Topic::new("cpu".into(), 0).is_err());
    assert!(Task::restore(1, "build".into(), TaskState::Running, vec![]).is_err());
    assert!(Task::restore(1, "build".into(), TaskState::Registered, vec!["cpu".into()]).is_err());
    assert!(
        Task::restore(
            1,
            "build".into(),
            TaskState::Queued,
            vec!["cpu".into(), "cpu".into()]
        )
        .is_err()
    );

    let mut task = Task::new(1, "build".into()).unwrap();

    assert!(task.add_topic("9bad topic").is_err());
    assert_eq!(task.state(), TaskState::Registered);
    assert!(task.topics().is_empty());
}

#[test]
fn capacity_cannot_be_reduced_below_existing_ownership() {
    let topic = Topic::new("cpu".into(), 2).unwrap();

    assert!(topic.require_capacity_for(2).is_ok());
    assert!(topic.require_capacity_for(3).is_err());
}

#[test]
fn autocreate_defaults_and_independent_switches_have_one_domain_owner() {
    let defaults = AutoCreatePolicy::default();

    assert!(defaults.require_task_creation("job").is_err());
    assert!(defaults.require_topic_creation("cpu").is_err());

    for (tasks, topics) in [(false, true), (true, false), (true, true)] {
        let policy = AutoCreatePolicy { tasks, topics };

        assert_eq!(policy.require_task_creation("job").is_ok(), tasks);
        assert_eq!(policy.require_topic_creation("cpu").is_ok(), topics);
    }
}
