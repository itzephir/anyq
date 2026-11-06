//! Queue use cases. SQL only implements the queries selected here.
pub mod ports;
#[cfg(test)]
mod tests;
use anyq_queue_api::*;
use ports::*;

pub struct Dispatch<S, C, T> {
    store: S,
    clock: C,
    tokens: T,
}
impl<S, C, T> Dispatch<S, C, T> {
    pub fn new(store: S, clock: C, tokens: T) -> Self {
        Self {
            store,
            clock,
            tokens,
        }
    }
}
fn topic(records: &mut dyn Records, name: &str) -> Result<Topic> {
    records
        .topic(name)?
        .ok_or_else(|| Error::NotFound(format!("topic '{name}' does not exist")))
}
fn group(records: &mut dyn Records, topic_name: &str, name: &str) -> Result<()> {
    if !records.groups(topic_name)?.iter().any(|g| g.name == name) {
        return Err(Error::NotFound(format!(
            "group '{name}' does not exist in '{topic_name}'"
        )));
    }
    Ok(())
}
fn recover(records: &mut dyn Records, topic_name: Option<String>, now: i64) -> Result<()> {
    let expired = records.deliveries(&DeliveryFilter {
        topic: topic_name,
        state: Some(State::Running),
        expired_at: Some(now),
        ..Default::default()
    })?;
    for mut d in expired {
        if d.expire(now) {
            records.put_delivery(&d)?;
        }
    }
    Ok(())
}
fn publish(records: &mut dyn Records, request: &Publish, now: i64) -> Result<Message> {
    request.validate()?;
    topic(records, &request.topic)?;
    if let Some(key) = &request.key {
        if let Some(existing) = records.message_by_key(&request.topic, key)? {
            return Ok(existing);
        }
    }
    let message = records.insert_message(request, now)?;
    for g in records.groups(&message.topic)? {
        records.put_delivery(&Delivery::pending(&message, g.name))?;
    }
    Ok(message)
}
fn delivery(records: &mut dyn Records, id: i64, group: String) -> Result<Delivery> {
    records
        .deliveries(&DeliveryFilter {
            message_id: Some(id),
            group: Some(group),
            ..Default::default()
        })?
        .into_iter()
        .next()
        .ok_or_else(|| Error::NotFound("delivery does not exist".into()))
}
fn view(records: &mut dyn Records, message: Message, group: Option<String>) -> Result<MessageView> {
    let deliveries = records.deliveries(&DeliveryFilter {
        message_id: Some(message.id),
        group,
        ..Default::default()
    })?;
    Ok(MessageView {
        message,
        deliveries,
    })
}
impl<S: Store, C: Clock, T: Tokens> Queue for Dispatch<S, C, T> {
    fn execute(&mut self, command: Command) -> Result<Output> {
        let clock = &self.clock;
        let tokens = &mut self.tokens;
        self.store.transaction(|records| {
            // Sample time only after acquiring the transaction's write lock.
            let now = clock.now_ms();
            match command {
                Command::CreateTopic(t) => {
                    let t = Topic::new(t.name, t.capacity)?;
                    if let Some(existing) = records.topic(&t.name)? {
                        if existing != t {
                            return Err(Error::Conflict(
                                "topic exists with another capacity; use topic capacity".into(),
                            ));
                        }
                    } else {
                        records.put_topic(&t)?;
                    }
                    Ok(Output::Topic(t))
                }
                Command::SetCapacity(t) => {
                    let t = Topic::new(t.name, t.capacity)?;
                    topic(records, &t.name)?;
                    records.put_topic(&t)?;
                    Ok(Output::Topic(t))
                }
                Command::Topics => Ok(Output::Topics(records.topics()?)),
                Command::CreateGroup(g) => {
                    validate_name(&g.name)?;
                    topic(records, &g.topic)?;
                    if !records.groups(&g.topic)?.contains(&g) {
                        records.insert_group(&g)?;
                        records.seed_group(&g)?;
                    }
                    Ok(Output::Group(g))
                }
                Command::Groups { topic: name } => {
                    topic(records, &name)?;
                    Ok(Output::Groups(records.groups(&name)?))
                }
                Command::Publish(request) => {
                    Ok(Output::Published(publish(records, &request, now)?))
                }
                Command::Claim(request) => {
                    validate_name(&request.consumer)?;
                    deadline(now, request.lease_ms)?;
                    let t = topic(records, &request.topic)?;
                    group(records, &t.name, &request.group)?;
                    recover(records, Some(t.name.clone()), now)?;
                    let active = records.count_deliveries(&DeliveryFilter {
                        topic: Some(t.name.clone()),
                        state: Some(State::Running),
                        ..Default::default()
                    })?;
                    if active >= u64::from(t.capacity) {
                        return Ok(Output::Claimed(None));
                    }
                    let ready = records.deliveries(&DeliveryFilter {
                        topic: Some(t.name),
                        group: Some(request.group),
                        state: Some(State::Pending),
                        ready_at: Some(now),
                        limit: Some(1),
                        ..Default::default()
                    })?;
                    let Some(mut d) = ready.into_iter().next() else {
                        return Ok(Output::Claimed(None));
                    };
                    d.claim(request.consumer, tokens.next(), now, request.lease_ms)?;
                    records.put_delivery(&d)?;
                    let message = records
                        .message(d.message_id)?
                        .ok_or_else(|| Error::Storage("delivery without message".into()))?;
                    Ok(Output::Claimed(Some(Assignment {
                        message,
                        delivery: d,
                    })))
                }
                Command::Renew { lease, lease_ms } => {
                    let mut d = delivery(records, lease.message_id, lease.group)?;
                    d.renew(&lease.token, now, lease_ms)?;
                    records.put_delivery(&d)?;
                    Ok(Output::Delivery(d))
                }
                Command::Ack {
                    lease,
                    result,
                    next,
                } => {
                    let mut d = delivery(records, lease.message_id, lease.group)?;
                    d.ack(&lease.token, now, result)?;
                    let next = next.map(|r| publish(records, &r, now)).transpose()?;
                    records.put_delivery(&d)?;
                    Ok(Output::Acked { delivery: d, next })
                }
                Command::Nack {
                    lease,
                    delay_ms,
                    reason,
                } => {
                    let mut d = delivery(records, lease.message_id, lease.group)?;
                    d.nack(&lease.token, now, delay_ms, reason)?;
                    records.put_delivery(&d)?;
                    Ok(Output::Delivery(d))
                }
                Command::Retry { message_id, group } => {
                    let message = records
                        .message(message_id)?
                        .ok_or_else(|| Error::NotFound("message does not exist".into()))?;
                    recover(records, Some(message.topic), now)?;
                    let mut d = delivery(records, message_id, group)?;
                    d.retry(now)?;
                    records.put_delivery(&d)?;
                    Ok(Output::Delivery(d))
                }
                Command::Inspect { message_id } => {
                    let message = records
                        .message(message_id)?
                        .ok_or_else(|| Error::NotFound("message does not exist".into()))?;
                    recover(records, Some(message.topic.clone()), now)?;
                    Ok(Output::Message(view(records, message, None)?))
                }
                Command::List {
                    topic: name,
                    group: g,
                    after,
                    limit,
                } => {
                    if after < 0 || !(1..=1000).contains(&limit) {
                        return Err(Error::Invalid(
                            "after must be nonnegative; limit must be in 1..1000".into(),
                        ));
                    }
                    if let Some(name) = &name {
                        topic(records, name)?;
                        if let Some(g) = &g {
                            group(records, name, g)?;
                        }
                    }
                    if name.is_none() && g.is_some() {
                        return Err(Error::Invalid("group filter requires a topic".into()));
                    }
                    recover(records, name.clone(), now)?;
                    let messages = records.messages(name.as_deref(), after, limit)?;
                    let views = messages
                        .into_iter()
                        .map(|m| view(records, m, g.clone()))
                        .collect::<Result<_>>()?;
                    Ok(Output::Messages(views))
                }
            }
        })
    }
}
