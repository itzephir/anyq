use super::*;
use std::{cell::Cell, rc::Rc};

#[derive(Clone, Default)]
struct Memory {
    topics: Vec<Topic>,
    groups: Vec<ConsumerGroup>,
    messages: Vec<Message>,
    deliveries: Vec<Delivery>,
    fail_save: bool,
}
impl Memory {
    fn select(&self, f: &DeliveryFilter) -> Vec<Delivery> {
        self.deliveries
            .iter()
            .filter(|d| {
                let m = self.messages.iter().find(|m| m.id == d.message_id).unwrap();
                f.topic.as_ref().is_none_or(|t| t == &m.topic)
                    && f.group.as_ref().is_none_or(|g| g == &d.group)
                    && f.message_id.is_none_or(|id| id == d.message_id)
                    && f.state.is_none_or(|s| s == d.state)
                    && f.ready_at.is_none_or(|t| d.available_at <= t)
                    && f.expired_at
                        .is_none_or(|t| d.lease_until.is_some_and(|u| u <= t))
            })
            .cloned()
            .collect()
    }
}
impl Store for Memory {
    fn transaction<T>(
        &mut self,
        operation: impl FnOnce(&mut dyn Records) -> Result<T>,
    ) -> Result<T> {
        let mut staged = self.clone();
        let result = operation(&mut staged)?;
        *self = staged;
        Ok(result)
    }
}
impl Records for Memory {
    fn topics(&mut self) -> Result<Vec<Topic>> {
        Ok(self.topics.clone())
    }
    fn topic(&mut self, name: &str) -> Result<Option<Topic>> {
        Ok(self.topics.iter().find(|t| t.name == name).cloned())
    }
    fn put_topic(&mut self, t: &Topic) -> Result<()> {
        self.topics.retain(|x| x.name != t.name);
        self.topics.push(t.clone());
        Ok(())
    }
    fn groups(&mut self, topic: &str) -> Result<Vec<ConsumerGroup>> {
        Ok(self
            .groups
            .iter()
            .filter(|g| g.topic == topic)
            .cloned()
            .collect())
    }
    fn insert_group(&mut self, g: &ConsumerGroup) -> Result<()> {
        self.groups.push(g.clone());
        Ok(())
    }
    fn seed_group(&mut self, g: &ConsumerGroup) -> Result<()> {
        for m in self.messages.clone().iter().filter(|m| m.topic == g.topic) {
            self.put_delivery(&Delivery::pending(m, g.name.clone()))?;
        }
        Ok(())
    }
    fn message(&mut self, id: i64) -> Result<Option<Message>> {
        Ok(self.messages.iter().find(|m| m.id == id).cloned())
    }
    fn message_by_key(&mut self, topic: &str, key: &str) -> Result<Option<Message>> {
        Ok(self
            .messages
            .iter()
            .find(|m| m.topic == topic && m.key.as_deref() == Some(key))
            .cloned())
    }
    fn insert_message(&mut self, p: &Publish, now: i64) -> Result<Message> {
        let m = Message {
            id: self.messages.len() as i64 + 1,
            topic: p.topic.clone(),
            payload: p.payload.clone(),
            key: p.key.clone(),
            max_attempts: p.max_attempts,
            created_at: now,
        };
        self.messages.push(m.clone());
        Ok(m)
    }
    fn messages(&mut self, topic: Option<&str>, after: i64, limit: u32) -> Result<Vec<Message>> {
        Ok(self
            .messages
            .iter()
            .filter(|m| m.id > after && topic.is_none_or(|t| t == m.topic))
            .take(limit as usize)
            .cloned()
            .collect())
    }
    fn deliveries(&mut self, f: &DeliveryFilter) -> Result<Vec<Delivery>> {
        let mut ds = self.select(f);
        ds.sort_by(|a, b| (a.message_id, &a.group).cmp(&(b.message_id, &b.group)));
        if let Some(limit) = f.limit {
            ds.truncate(limit as usize);
        }
        Ok(ds)
    }
    fn count_deliveries(&mut self, f: &DeliveryFilter) -> Result<u64> {
        Ok(self.select(f).len() as u64)
    }
    fn put_delivery(&mut self, d: &Delivery) -> Result<()> {
        if self.fail_save {
            return Err(Error::Storage("injected failure".into()));
        }
        self.deliveries
            .retain(|x| !(x.message_id == d.message_id && x.group == d.group));
        self.deliveries.push(d.clone());
        Ok(())
    }
}
struct TestClock(Rc<Cell<i64>>);
impl Clock for TestClock {
    fn now_ms(&self) -> i64 {
        self.0.get()
    }
}
#[derive(Default)]
struct Sequence(u32);
impl Tokens for Sequence {
    fn next(&mut self) -> String {
        self.0 += 1;
        self.0.to_string()
    }
}
fn setup() -> (Dispatch<Memory, TestClock, Sequence>, Rc<Cell<i64>>) {
    let now = Rc::new(Cell::new(0));
    let mut q = Dispatch::new(
        Memory::default(),
        TestClock(now.clone()),
        Sequence::default(),
    );
    q.execute(Command::CreateTopic(
        Topic::new("builds".into(), 1).unwrap(),
    ))
    .unwrap();
    q.execute(Command::CreateGroup(ConsumerGroup {
        topic: "builds".into(),
        name: "builders".into(),
    }))
    .unwrap();
    (q, now)
}
fn request(topic: &str) -> Publish {
    Publish {
        topic: topic.into(),
        payload: "{}".into(),
        key: None,
        max_attempts: 2,
    }
}
fn claim(q: &mut dyn Queue, group: &str) -> Option<Assignment> {
    let Output::Claimed(a) = q
        .execute(Command::Claim(Claim {
            topic: "builds".into(),
            group: group.into(),
            consumer: "worker".into(),
            lease_ms: 100,
        }))
        .unwrap()
    else {
        panic!()
    };
    a
}
#[test]
fn groups_replay_history_and_complete_independently() {
    let (mut q, _) = setup();
    q.execute(Command::Publish(request("builds"))).unwrap();
    let a = claim(&mut q, "builders").unwrap();
    q.execute(Command::Ack {
        lease: a.lease(),
        result: None,
        next: None,
    })
    .unwrap();
    q.execute(Command::CreateGroup(ConsumerGroup {
        topic: "builds".into(),
        name: "audit".into(),
    }))
    .unwrap();
    let audit = claim(&mut q, "audit").unwrap();
    assert_eq!(a.message.id, audit.message.id);
    assert_eq!(audit.delivery.attempts, 1);
    assert!(claim(&mut q, "builders").is_none());
}
#[test]
fn capacity_is_shared_between_groups_and_expired_token_is_fenced() {
    let (mut q, now) = setup();
    q.execute(Command::CreateGroup(ConsumerGroup {
        topic: "builds".into(),
        name: "audit".into(),
    }))
    .unwrap();
    q.execute(Command::Publish(request("builds"))).unwrap();
    let a = claim(&mut q, "builders").unwrap();
    assert!(claim(&mut q, "audit").is_none());
    now.set(100);
    let b = claim(&mut q, "builders").unwrap();
    assert_ne!(a.delivery.token, b.delivery.token);
    assert_eq!(
        q.execute(Command::Ack {
            lease: a.lease(),
            result: None,
            next: None
        }),
        Err(Error::LeaseLost)
    );
}
#[test]
fn delayed_retry_does_not_block_next_ready_message() {
    let (mut q, _) = setup();
    q.execute(Command::Publish(request("builds"))).unwrap();
    q.execute(Command::Publish(request("builds"))).unwrap();
    let a = claim(&mut q, "builders").unwrap();
    q.execute(Command::Nack {
        lease: a.lease(),
        delay_ms: 50,
        reason: "retry".into(),
    })
    .unwrap();
    let b = claim(&mut q, "builders").unwrap();
    assert!(b.message.id > a.message.id);
}
#[test]
fn handoff_rolls_back_publication_if_delivery_save_fails() {
    let (mut q, _) = setup();
    q.execute(Command::CreateTopic(
        Topic::new("emulator".into(), 1).unwrap(),
    ))
    .unwrap();
    q.execute(Command::Publish(request("builds"))).unwrap();
    let a = claim(&mut q, "builders").unwrap();
    q.store.fail_save = true;
    assert!(
        q.execute(Command::Ack {
            lease: a.lease(),
            result: None,
            next: Some(request("emulator"))
        })
        .is_err()
    );
    assert_eq!(q.store.messages.len(), 1);
    assert_eq!(q.store.deliveries[0].state, State::Running);
}
#[test]
fn producer_key_returns_original_payload_without_new_deliveries() {
    let (mut q, _) = setup();
    let mut r = request("builds");
    r.key = Some("key".into());
    let first = q.execute(Command::Publish(r.clone())).unwrap();
    r.payload = "changed".into();
    assert_eq!(first, q.execute(Command::Publish(r)).unwrap());
    assert_eq!(q.store.deliveries.len(), 1);
}
