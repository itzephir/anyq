mod memory;
mod memory_store;
mod sequence;
mod test_clock;

use crate::*;
use anyq_queue_api::*;
use memory_store::MemoryStore;
use sequence::Sequence;
use test_clock::TestClock;

fn setup() -> MemoryStore {
    let store = MemoryStore::default();

    CreateTopicInteractor::new(store.clone())
        .create(Topic::new("builds".into(), 1).unwrap())
        .unwrap();

    CreateConsumerGroupInteractor::new(store.clone())
        .create(ConsumerGroup {
            topic: "builds".into(),
            name: "builders".into(),
        })
        .unwrap();

    store
}

fn request(topic: &str) -> Publish {
    Publish {
        topic: topic.into(),
        payload: "{}".into(),
        key: None,
        max_attempts: 2,
    }
}

fn claim(
    store: &MemoryStore,
    clock: &TestClock,
    tokens: &Sequence,
    group: &str,
) -> Option<Assignment> {
    ClaimDeliveryInteractor::new(store.clone(), clock.clone(), tokens.clone())
        .claim(Claim {
            topic: "builds".into(),
            group: group.into(),
            consumer: "worker".into(),
            lease_ms: 100,
        })
        .unwrap()
}

#[test]
fn groups_replay_history_and_complete_independently() {
    let store = setup();
    let clock = TestClock::default();
    let tokens = Sequence::default();

    PublishMessageInteractor::new(store.clone(), clock.clone())
        .publish(request("builds"))
        .unwrap();

    let a = claim(&store, &clock, &tokens, "builders").unwrap();

    CompleteDeliveryInteractor::new(store.clone(), clock.clone())
        .complete(CompleteDeliveryRequest {
            lease: a.lease(),
            result: None,
            next: None,
        })
        .unwrap();

    CreateConsumerGroupInteractor::new(store.clone())
        .create(ConsumerGroup {
            topic: "builds".into(),
            name: "audit".into(),
        })
        .unwrap();

    let audit = claim(&store, &clock, &tokens, "audit").unwrap();

    assert_eq!(a.message.id, audit.message.id);
    assert_eq!(audit.delivery.attempts, 1);
    assert!(claim(&store, &clock, &tokens, "builders").is_none());
}

#[test]
fn capacity_is_shared_between_groups_and_expired_token_is_fenced() {
    let store = setup();
    let clock = TestClock::default();
    let tokens = Sequence::default();

    CreateConsumerGroupInteractor::new(store.clone())
        .create(ConsumerGroup {
            topic: "builds".into(),
            name: "audit".into(),
        })
        .unwrap();

    PublishMessageInteractor::new(store.clone(), clock.clone())
        .publish(request("builds"))
        .unwrap();

    let a = claim(&store, &clock, &tokens, "builders").unwrap();

    assert!(claim(&store, &clock, &tokens, "audit").is_none());

    clock.0.set(100);

    let b = claim(&store, &clock, &tokens, "builders").unwrap();

    assert_ne!(a.delivery.token, b.delivery.token);
    assert_eq!(
        CompleteDeliveryInteractor::new(store, clock).complete(CompleteDeliveryRequest {
            lease: a.lease(),
            result: None,
            next: None
        }),
        Err(Error::LeaseLost)
    );
}

#[test]
fn delayed_retry_does_not_block_next_ready_message() {
    let store = setup();
    let clock = TestClock::default();
    let tokens = Sequence::default();
    let mut publisher = PublishMessageInteractor::new(store.clone(), clock.clone());

    publisher.publish(request("builds")).unwrap();
    publisher.publish(request("builds")).unwrap();

    let a = claim(&store, &clock, &tokens, "builders").unwrap();

    RejectDeliveryInteractor::new(store.clone(), clock.clone())
        .reject(RejectDeliveryRequest {
            lease: a.lease(),
            delay_ms: 50,
            reason: "retry".into(),
        })
        .unwrap();

    let b = claim(&store, &clock, &tokens, "builders").unwrap();

    assert!(b.message.id > a.message.id);
}

#[test]
fn handoff_rolls_back_publication_if_delivery_save_fails() {
    let store = setup();
    let clock = TestClock::default();
    let tokens = Sequence::default();

    CreateTopicInteractor::new(store.clone())
        .create(Topic::new("emulator".into(), 1).unwrap())
        .unwrap();

    PublishMessageInteractor::new(store.clone(), clock.clone())
        .publish(request("builds"))
        .unwrap();

    let a = claim(&store, &clock, &tokens, "builders").unwrap();

    store.data.borrow_mut().fail_save = true;

    assert!(
        CompleteDeliveryInteractor::new(store.clone(), clock)
            .complete(CompleteDeliveryRequest {
                lease: a.lease(),
                result: None,
                next: Some(request("emulator"))
            })
            .is_err()
    );
    assert_eq!(store.data.borrow().messages.len(), 1);
    assert_eq!(store.data.borrow().deliveries[0].state, State::Running);
}

#[test]
fn producer_key_returns_original_payload_without_new_deliveries() {
    let store = setup();
    let mut publisher = PublishMessageInteractor::new(store.clone(), TestClock::default());
    let mut request = request("builds");
    request.key = Some("key".into());

    let first = publisher.publish(request.clone()).unwrap();

    request.payload = "changed".into();

    assert_eq!(first, publisher.publish(request).unwrap());
    assert_eq!(store.data.borrow().deliveries.len(), 1);
}
