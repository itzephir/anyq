mod sequence;
mod test_clock;
use super::*;
use anyq_queue_api::*;
use anyq_queue_dispatch::*;
use sequence::Sequence;
use std::{
    path::Path,
    sync::{Arc, Barrier},
    thread,
};
use test_clock::TestClock;
fn publish(store: &Sqlite, clock: &TestClock, topic: &str, key: &str) -> Message {
    PublishMessageInteractor::new(store.clone(), clock.clone())
        .publish(Publish {
            topic: topic.into(),
            payload: "{}".into(),
            key: Some(key.into()),
            max_attempts: 2,
        })
        .unwrap()
}
fn group(store: &Sqlite, name: &str) {
    CreateConsumerGroupInteractor::new(store.clone())
        .create(ConsumerGroup {
            topic: "builds".into(),
            name: name.into(),
        })
        .unwrap();
}
fn claim(store: &Sqlite, clock: &TestClock, tokens: &Sequence, group: &str) -> Option<Assignment> {
    ClaimDeliveryInteractor::new(store.clone(), clock.clone(), tokens.clone())
        .claim(Claim {
            topic: "builds".into(),
            group: group.into(),
            consumer: "test".into(),
            lease_ms: 100,
        })
        .unwrap()
}
fn topic(store: &Sqlite, name: &str) {
    CreateTopicInteractor::new(store.clone())
        .create(Topic::new(name.into(), 1).unwrap())
        .unwrap();
}
fn list(store: &Sqlite, clock: &TestClock, topic: Option<String>) -> Vec<MessageView> {
    ListMessagesInteractor::new(store.clone(), clock.clone())
        .list(MessageQuery {
            topic,
            group: None,
            after: 0,
            limit: 100,
        })
        .unwrap()
}
#[test]
fn persistence_groups_expiry_and_atomic_handoff() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("q.db");
    let clock = TestClock::default();
    let tokens = Sequence::default();
    let store = Sqlite::open(&path).unwrap();
    topic(&store, "builds");
    group(&store, "builders");
    let published = publish(&store, &clock, "builds", "once");
    drop(store);
    let store = Sqlite::open(&path).unwrap();
    assert_eq!(published, publish(&store, &clock, "builds", "once"));
    let a = claim(&store, &clock, &tokens, "builders").unwrap();
    assert!(
        CompleteDeliveryInteractor::new(store.clone(), clock.clone())
            .complete(CompleteDeliveryRequest {
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
    RenewLeaseInteractor::new(store.clone(), clock.clone())
        .renew(a.lease(), 100)
        .unwrap();
    clock.0.set(100);
    let b = claim(&store, &clock, &tokens, "builders").unwrap();
    assert_eq!(b.delivery.attempts, 2);
    assert_eq!(
        CompleteDeliveryInteractor::new(store.clone(), clock.clone()).complete(
            CompleteDeliveryRequest {
                lease: a.lease(),
                result: None,
                next: None
            }
        ),
        Err(Error::LeaseLost)
    );
    topic(&store, "emulator");
    CompleteDeliveryInteractor::new(store.clone(), clock.clone())
        .complete(CompleteDeliveryRequest {
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
    group(&store, "audit");
    let audit = claim(&store, &clock, &tokens, "audit").unwrap();
    assert_eq!(a.message.id, audit.message.id);
    assert_eq!(audit.delivery.attempts, 1);
    assert_eq!(list(&store, &clock, Some("emulator".into())).len(), 1);
}
#[test]
fn sql_failure_rolls_back_both_sides_of_handoff() {
    let store = Sqlite::open(Path::new(":memory:")).unwrap();
    let clock = TestClock::default();
    store.connection.borrow().execute_batch("CREATE TRIGGER reject_ack BEFORE UPDATE ON deliveries WHEN NEW.state='succeeded' BEGIN SELECT RAISE(ABORT,'injected failure'); END;").unwrap();
    topic(&store, "builds");
    group(&store, "builders");
    publish(&store, &clock, "builds", "once");
    let a = claim(&store, &clock, &Sequence::default(), "builders").unwrap();
    assert!(
        CompleteDeliveryInteractor::new(store.clone(), clock.clone())
            .complete(CompleteDeliveryRequest {
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
    let messages = list(&store, &clock, None);
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].deliveries[0].state, State::Running);
}
#[test]
fn independent_connections_never_exceed_capacity() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("q.db");
    let store = Sqlite::open(&path).unwrap();
    let clock = TestClock::default();
    topic(&store, "builds");
    group(&store, "builders");
    for i in 0..8 {
        publish(&store, &clock, "builds", &i.to_string());
    }
    drop(store);
    let barrier = Arc::new(Barrier::new(8));
    let handles: Vec<_> = (0..8)
        .map(|_| {
            let path = path.clone();
            let barrier = barrier.clone();
            thread::spawn(move || {
                let store = Sqlite::open(&path).unwrap();
                barrier.wait();
                claim(
                    &store,
                    &TestClock::default(),
                    &Sequence::default(),
                    "builders",
                )
                .is_some()
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
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection.pragma_update(None, "user_version", 99).unwrap();
    drop(connection);
    assert!(Sqlite::open(&path).is_err());
}
