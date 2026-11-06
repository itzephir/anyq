use super::*;
use anyq_queue_dispatch::{
    Dispatch,
    ports::{Clock, Tokens},
};
use std::{
    cell::Cell,
    rc::Rc,
    sync::{Arc, Barrier},
    thread,
};
struct TestClock(Rc<Cell<i64>>);
impl Clock for TestClock {
    fn now_ms(&self) -> i64 {
        self.0.get()
    }
}
struct Sequence(u32);
impl Tokens for Sequence {
    fn next(&mut self) -> String {
        self.0 += 1;
        self.0.to_string()
    }
}
fn publish(topic: &str) -> Command {
    Command::Publish(Publish {
        topic: topic.into(),
        payload: "{}".into(),
        key: Some("once".into()),
        max_attempts: 2,
    })
}
fn group(name: &str) -> Command {
    Command::CreateGroup(ConsumerGroup {
        topic: "builds".into(),
        name: name.into(),
    })
}
fn claim(group: &str) -> Command {
    Command::Claim(Claim {
        topic: "builds".into(),
        group: group.into(),
        consumer: "test".into(),
        lease_ms: 100,
    })
}
fn assignment(out: Output) -> Assignment {
    let Output::Claimed(Some(a)) = out else {
        panic!()
    };
    a
}
#[test]
fn persistence_groups_expiry_and_atomic_handoff() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("q.db");
    let now = Rc::new(Cell::new(0));
    let open = || {
        Dispatch::new(
            Sqlite::open(&path).unwrap(),
            TestClock(now.clone()),
            Sequence(0),
        )
    };
    let mut q = open();
    q.execute(Command::CreateTopic(
        Topic::new("builds".into(), 1).unwrap(),
    ))
    .unwrap();
    q.execute(group("builders")).unwrap();
    let published = q.execute(publish("builds")).unwrap();
    drop(q);
    let mut q = open();
    assert_eq!(published, q.execute(publish("builds")).unwrap());
    let a = assignment(q.execute(claim("builders")).unwrap());
    assert!(
        q.execute(Command::Ack {
            lease: a.lease(),
            result: None,
            next: Some(Publish {
                topic: "missing".into(),
                payload: "{}".into(),
                key: None,
                max_attempts: 1
            })
        })
        .is_err()
    );
    q.execute(Command::Renew {
        lease: a.lease(),
        lease_ms: 100,
    })
    .unwrap();
    now.set(100);
    let b = assignment(q.execute(claim("builders")).unwrap());
    assert_eq!(b.delivery.attempts, 2);
    // Sequence restarted only on database reopen; within this process tokens differ.
    assert_eq!(
        q.execute(Command::Ack {
            lease: a.lease(),
            result: None,
            next: None
        }),
        Err(Error::LeaseLost)
    );
    q.execute(Command::CreateTopic(
        Topic::new("emulator".into(), 1).unwrap(),
    ))
    .unwrap();
    q.execute(Command::Ack {
        lease: b.lease(),
        result: Some("{}".into()),
        next: Some(Publish {
            topic: "emulator".into(),
            payload: "{}".into(),
            key: None,
            max_attempts: 1,
        }),
    })
    .unwrap();
    q.execute(group("audit")).unwrap();
    let audit = assignment(q.execute(claim("audit")).unwrap());
    assert_eq!(a.message.id, audit.message.id);
    assert_eq!(audit.delivery.attempts, 1);
    let Output::Messages(messages) = q
        .execute(Command::List {
            topic: Some("emulator".into()),
            group: None,
            after: 0,
            limit: 100,
        })
        .unwrap()
    else {
        panic!()
    };
    assert_eq!(messages.len(), 1);
}
#[test]
fn sql_failure_rolls_back_both_sides_of_handoff() {
    let store = Sqlite::open(Path::new(":memory:")).unwrap();
    let now = Rc::new(Cell::new(0));
    store.connection.execute_batch("CREATE TRIGGER reject_ack BEFORE UPDATE ON deliveries WHEN NEW.state='succeeded' BEGIN SELECT RAISE(ABORT,'injected failure'); END;").unwrap();
    let mut q = Dispatch::new(store, TestClock(now), Sequence(0));
    q.execute(Command::CreateTopic(
        Topic::new("builds".into(), 1).unwrap(),
    ))
    .unwrap();
    q.execute(group("builders")).unwrap();
    q.execute(publish("builds")).unwrap();
    let a = assignment(q.execute(claim("builders")).unwrap());
    assert!(
        q.execute(Command::Ack {
            lease: a.lease(),
            result: None,
            next: Some(Publish {
                topic: "builds".into(),
                payload: "{}".into(),
                key: Some("next".into()),
                max_attempts: 1
            })
        })
        .is_err()
    );
    let Output::Messages(messages) = q
        .execute(Command::List {
            topic: None,
            group: None,
            after: 0,
            limit: 100,
        })
        .unwrap()
    else {
        panic!()
    };
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].deliveries[0].state, State::Running);
}
#[test]
fn independent_connections_never_exceed_capacity() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("q.db");
    let mut q = Dispatch::new(
        Sqlite::open(&path).unwrap(),
        TestClock(Rc::new(Cell::new(0))),
        Sequence(0),
    );
    q.execute(Command::CreateTopic(
        Topic::new("builds".into(), 1).unwrap(),
    ))
    .unwrap();
    q.execute(group("builders")).unwrap();
    for i in 0..8 {
        q.execute(Command::Publish(Publish {
            topic: "builds".into(),
            payload: "{}".into(),
            key: Some(i.to_string()),
            max_attempts: 2,
        }))
        .unwrap();
    }
    drop(q);
    let barrier = Arc::new(Barrier::new(8));
    let handles: Vec<_> = (0..8)
        .map(|_| {
            let path = path.clone();
            let barrier = barrier.clone();
            thread::spawn(move || {
                let mut q = Dispatch::new(
                    Sqlite::open(&path).unwrap(),
                    TestClock(Rc::new(Cell::new(0))),
                    Sequence(0),
                );
                barrier.wait();
                matches!(
                    q.execute(claim("builders")).unwrap(),
                    Output::Claimed(Some(_))
                )
            })
        })
        .collect();
    assert_eq!(
        handles
            .into_iter()
            .map(|h| u32::from(h.join().unwrap()))
            .sum::<u32>(),
        1
    );
}
#[test]
fn future_schema_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("q.db");
    let connection = Connection::open(&path).unwrap();
    connection.pragma_update(None, "user_version", 99).unwrap();
    drop(connection);
    assert!(Sqlite::open(&path).is_err());
}
