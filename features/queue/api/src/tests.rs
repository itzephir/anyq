use super::*;
fn delivery(max_attempts: u32) -> Delivery {
    Delivery::pending(
        &Message {
            id: 1,
            topic: "builds".into(),
            payload: "{}".into(),
            key: None,
            max_attempts,
            created_at: 0,
        },
        "builders".into(),
    )
}
#[test]
fn expired_and_replaced_tokens_cannot_complete() {
    let mut d = delivery(3);
    d.claim("first".into(), "old".into(), 0, 100).unwrap();
    assert_eq!(d.ack("old", 100, None), Err(Error::LeaseLost));
    assert!(d.expire(100));
    d.claim("second".into(), "new".into(), 100, 100).unwrap();
    assert_eq!(d.ack("old", 101, None), Err(Error::LeaseLost));
    d.ack("new", 101, None).unwrap();
    assert_eq!(d.state, State::Succeeded);
    assert_eq!(d.ack("new", 102, None), Err(Error::LeaseLost));
}
#[test]
fn last_attempt_fails_and_explicit_retry_resets_budget() {
    let mut d = delivery(1);
    d.claim("builder".into(), "token".into(), 0, 10).unwrap();
    assert!(d.expire(10));
    assert_eq!(d.state, State::Failed);
    d.retry(20).unwrap();
    assert_eq!(d.attempts, 0);
    assert_eq!(d.available_at, 20);
}
#[test]
fn renewal_never_shortens_lease_and_delayed_retry_is_not_ready() {
    let mut d = delivery(2);
    d.claim("builder".into(), "token".into(), 0, 100).unwrap();
    d.renew("token", 5, 10).unwrap();
    assert_eq!(d.lease_until, Some(100));
    d.nack("token", 10, 50, "failure".into()).unwrap();
    assert!(d.claim("builder".into(), "next".into(), 59, 100).is_err());
    d.claim("builder".into(), "next".into(), 60, 100).unwrap();
}
#[test]
fn invalid_inputs_and_overflow_are_rejected() {
    assert!(Topic::new("bad/name".into(), 1).is_err());
    assert!(Topic::new("ok".into(), 0).is_err());
    assert!(deadline(i64::MAX, 1).is_err());
    assert!(deadline(0, 0).is_err());
}
