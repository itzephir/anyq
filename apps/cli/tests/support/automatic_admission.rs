use crate::support::{process::TestProcess, queue::Queue};
use std::process::Stdio;

#[test]
fn completion_and_capacity_promote_tasks_without_any_wait_process() {
    let q = Queue::new();
    q.topic("cpu");
    q.topic("repo");
    q.task("owner", &["cpu", "repo"]);
    q.task("second", &["cpu", "repo"]);
    q.task("third", &["cpu", "repo"]);

    assert_eq!(q.state("owner"), "running");
    assert_eq!(q.state("second"), "queued");

    q.call(&["task", "complete", "owner"]);

    assert_eq!(q.state("second"), "running");
    assert_eq!(q.state("third"), "queued");
    q.call(&["topic", "capacity", "cpu", "2"]);
    assert_eq!(q.state("third"), "queued");
    assert_eq!(q.usage("cpu"), 1);

    q.call(&["topic", "capacity", "repo", "2"]);

    assert_eq!(q.state("third"), "running");
    assert_eq!(q.usage("cpu"), 2);
    assert_eq!(q.usage("repo"), 2);
}

#[test]
fn batch_task_links_attach_every_resource_before_starting_in_both_orders() {
    for arguments in [
        vec!["task", "link", "job", "cpu", "repo", "cpu"],
        vec!["task", "job", "link", "cpu", "repo", "cpu"],
    ] {
        let q = Queue::new();
        q.topic("cpu");
        q.topic("repo");
        q.task("job", &[]);

        q.call(&arguments);

        assert_eq!(q.state("job"), "running");
        assert_eq!(q.usage("cpu"), 1);
        assert_eq!(q.usage("repo"), 1);
        assert_eq!(
            q.call(&["task", "job"]),
            "task: job\nstatus: running\ntopics:\n- cpu\n- repo"
        );
    }
}

#[test]
fn batch_topic_links_use_registration_order_and_complete_each_task_when_requested() {
    for prefix in [vec!["topic", "link", "cpu"], vec!["topic", "cpu", "link"]] {
        for complete in [false, true] {
            let q = Queue::new();
            q.topic("cpu");
            q.task("z-first", &[]);
            q.task("a-second", &[]);
            q.task("third", &[]);
            let mut arguments = prefix.clone();
            arguments.extend(["third", "a-second", "z-first", "a-second"]);

            if complete {
                arguments.push("-wc");
            }

            q.call(&arguments);

            assert_eq!(
                q.state("z-first"),
                if complete { "completed" } else { "running" }
            );
            for name in ["a-second", "third"] {
                assert_eq!(q.state(name), if complete { "completed" } else { "queued" });
            }
            assert_eq!(q.usage("cpu"), i64::from(!complete));
        }
    }
}

#[test]
fn invalid_batches_preserve_prior_memberships_and_roll_back_autocreate() {
    let q = Queue::new();
    q.topic("cpu");
    q.topic("repo");
    q.task("first", &[]);
    q.task("second", &[]);

    for arguments in [
        vec!["topic", "link", "cpu", "first", "second", "missing"],
        vec!["task", "link", "first", "cpu", "repo", "missing"],
    ] {
        assert_eq!(q.raw(&arguments).status.code(), Some(2));
        assert_eq!(q.state("first"), "registered");
        assert_eq!(q.state("second"), "registered");
        assert_eq!(q.usage("cpu"), 0);
        assert_eq!(q.usage("repo"), 0);
    }

    q.call(&["config", "task.autocreate", "true"]);
    q.call(&["config", "topic.autocreate", "true"]);
    q.call(&["task", "complete", "second", "-f"]);

    assert_eq!(
        q.raw(&["topic", "link", "transient", "new-task", "first", "second"])
            .status
            .code(),
        Some(2)
    );
    assert_eq!(
        q.raw(&["task", "link", "other-new", "transient", "1-invalid"])
            .status
            .code(),
        Some(2)
    );
    assert_eq!(q.call(&["task", "list"]), "- first\n- second");
    assert_eq!(q.call(&["topic", "list"]), "- cpu\n- repo");
    assert_eq!(q.state("first"), "registered");
}

#[test]
fn concurrent_registrations_cannot_overbook_multiple_resources() {
    let q = Queue::new();
    q.topic("cpu");
    q.topic("repo");
    let processes: Vec<_> = ["a", "b", "c", "d"]
        .into_iter()
        .map(|name| {
            TestProcess::spawn(
                q.cmd(&["task", "create", name, "-t", "cpu", "-t", "repo"])
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped()),
            )
        })
        .collect();

    for child in processes {
        assert!(child.output().status.success());
    }

    assert_eq!(q.usage("cpu"), 1);
    assert_eq!(q.usage("repo"), 1);
    assert_eq!(
        ["a", "b", "c", "d"]
            .iter()
            .filter(|name| q.state(name) == "running")
            .count(),
        1
    );

    let connection = rusqlite::Connection::open(&q.db).unwrap();
    let mut query = connection
        .prepare("SELECT name FROM tasks ORDER BY id")
        .unwrap();
    let names = query
        .query_map([], |row| row.get::<_, String>(0))
        .unwrap()
        .collect::<rusqlite::Result<Vec<_>>>()
        .unwrap();

    for name in names {
        assert_eq!(q.state(&name), "running");
        q.call(&["task", "complete", &name]);
    }

    assert_eq!(q.usage("cpu"), 0);
    assert_eq!(q.usage("repo"), 0);
}
