//! Pure queue contracts and rules. No IO, serialization or database types.
use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Invalid(String),
    NotFound(String),
    Conflict(String),
    LeaseLost,
    Storage(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(s) | Self::NotFound(s) | Self::Conflict(s) | Self::Storage(s) => {
                f.write_str(s)
            }
            Self::LeaseLost => f.write_str("lease expired, replaced, or already completed"),
        }
    }
}
impl std::error::Error for Error {}

pub fn validate_name(value: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
    {
        return Err(Error::Invalid(
            "names must contain 1..128 ASCII letters, digits, '-', '_' or '.'".into(),
        ));
    }
    Ok(())
}

pub fn deadline(now: i64, duration_ms: i64) -> Result<i64> {
    if !(1..=86_400_000).contains(&duration_ms) {
        return Err(Error::Invalid(
            "duration must be in 1..86400000 milliseconds".into(),
        ));
    }
    now.checked_add(duration_ms)
        .ok_or_else(|| Error::Invalid("timestamp overflow".into()))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Topic {
    pub name: String,
    pub capacity: u32,
}
impl Topic {
    pub fn new(name: String, capacity: u32) -> Result<Self> {
        validate_name(&name)?;
        if capacity == 0 {
            return Err(Error::Invalid("capacity must be positive".into()));
        }
        Ok(Self { name, capacity })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumerGroup {
    pub topic: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Publish {
    pub topic: String,
    /// Opaque application data. JSON is validated by the external controller.
    pub payload: String,
    pub key: Option<String>,
    pub max_attempts: u32,
}
impl Publish {
    pub fn validate(&self) -> Result<()> {
        validate_name(&self.topic)?;
        if self.max_attempts == 0 || self.max_attempts > 1_000_000 {
            return Err(Error::Invalid("max attempts must be in 1..1000000".into()));
        }
        if self
            .key
            .as_ref()
            .is_some_and(|k| k.is_empty() || k.len() > 512)
        {
            return Err(Error::Invalid(
                "idempotency key must contain 1..512 bytes".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub id: i64,
    pub topic: String,
    pub payload: String,
    pub key: Option<String>,
    pub max_attempts: u32,
    pub created_at: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Pending,
    Running,
    Succeeded,
    Failed,
}
impl State {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Delivery {
    pub message_id: i64,
    pub group: String,
    pub state: State,
    pub attempts: u32,
    pub max_attempts: u32,
    pub available_at: i64,
    pub consumer: Option<String>,
    pub token: Option<String>,
    pub lease_until: Option<i64>,
    pub finished_at: Option<i64>,
    pub result: Option<String>,
    pub last_error: Option<String>,
}
impl Delivery {
    pub fn pending(message: &Message, group: String) -> Self {
        Self {
            message_id: message.id,
            group,
            state: State::Pending,
            attempts: 0,
            max_attempts: message.max_attempts,
            available_at: message.created_at,
            consumer: None,
            token: None,
            lease_until: None,
            finished_at: None,
            result: None,
            last_error: None,
        }
    }
    pub fn claim(
        &mut self,
        consumer: String,
        token: String,
        now: i64,
        duration: i64,
    ) -> Result<()> {
        validate_name(&consumer)?;
        let until = deadline(now, duration)?;
        if self.state != State::Pending
            || self.available_at > now
            || self.attempts >= self.max_attempts
        {
            return Err(Error::Conflict("delivery is not ready".into()));
        }
        if token.is_empty() {
            return Err(Error::Invalid("lease token must not be empty".into()));
        }
        self.state = State::Running;
        self.attempts += 1;
        self.consumer = Some(consumer);
        self.token = Some(token);
        self.lease_until = Some(until);
        self.finished_at = None;
        Ok(())
    }
    pub fn require_lease(&self, token: &str, now: i64) -> Result<()> {
        if self.state != State::Running
            || self.token.as_deref() != Some(token)
            || self.lease_until.is_none_or(|t| t <= now)
        {
            return Err(Error::LeaseLost);
        }
        Ok(())
    }
    fn release(&mut self) {
        self.token = None;
        self.lease_until = None;
    }
    pub fn expire(&mut self, now: i64) -> bool {
        if self.state != State::Running || self.lease_until.is_none_or(|t| t > now) {
            return false;
        }
        self.fail_or_retry(now, "lease expired".into());
        true
    }
    fn fail_or_retry(&mut self, available_at: i64, reason: String) {
        self.release();
        self.last_error = Some(reason);
        self.available_at = available_at;
        self.state = if self.attempts >= self.max_attempts {
            State::Failed
        } else {
            State::Pending
        };
        self.finished_at = (self.state == State::Failed).then_some(available_at);
    }
    pub fn renew(&mut self, token: &str, now: i64, duration: i64) -> Result<()> {
        self.require_lease(token, now)?;
        self.lease_until = Some(deadline(now, duration)?.max(self.lease_until.unwrap_or(now)));
        Ok(())
    }
    pub fn ack(&mut self, token: &str, now: i64, result: Option<String>) -> Result<()> {
        self.require_lease(token, now)?;
        self.state = State::Succeeded;
        self.finished_at = Some(now);
        self.result = result;
        self.release();
        Ok(())
    }
    pub fn nack(&mut self, token: &str, now: i64, delay_ms: i64, reason: String) -> Result<()> {
        self.require_lease(token, now)?;
        if !(0..=86_400_000).contains(&delay_ms) {
            return Err(Error::Invalid(
                "retry delay must be in 0..86400000 milliseconds".into(),
            ));
        }
        let ready = now
            .checked_add(delay_ms)
            .ok_or_else(|| Error::Invalid("timestamp overflow".into()))?;
        self.fail_or_retry(ready, reason);
        if self.state == State::Failed {
            self.finished_at = Some(now);
        }
        Ok(())
    }
    pub fn retry(&mut self, now: i64) -> Result<()> {
        if self.state != State::Failed {
            return Err(Error::Conflict(
                "only failed deliveries can be retried".into(),
            ));
        }
        self.state = State::Pending;
        self.attempts = 0;
        self.available_at = now;
        self.consumer = None;
        self.finished_at = None;
        self.result = None;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lease {
    pub message_id: i64,
    pub group: String,
    pub token: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Claim {
    pub topic: String,
    pub group: String,
    pub consumer: String,
    pub lease_ms: i64,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assignment {
    pub message: Message,
    pub delivery: Delivery,
}
impl Assignment {
    pub fn lease(&self) -> Lease {
        Lease {
            message_id: self.message.id,
            group: self.delivery.group.clone(),
            token: self.delivery.token.clone().unwrap_or_default(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageView {
    pub message: Message,
    pub deliveries: Vec<Delivery>,
}

#[derive(Debug, Clone)]
pub enum Command {
    CreateTopic(Topic),
    SetCapacity(Topic),
    Topics,
    CreateGroup(ConsumerGroup),
    Groups {
        topic: String,
    },
    Publish(Publish),
    Claim(Claim),
    Renew {
        lease: Lease,
        lease_ms: i64,
    },
    Ack {
        lease: Lease,
        result: Option<String>,
        next: Option<Publish>,
    },
    Nack {
        lease: Lease,
        delay_ms: i64,
        reason: String,
    },
    Retry {
        message_id: i64,
        group: String,
    },
    Inspect {
        message_id: i64,
    },
    List {
        topic: Option<String>,
        group: Option<String>,
        after: i64,
        limit: u32,
    },
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Output {
    Topic(Topic),
    Topics(Vec<Topic>),
    Group(ConsumerGroup),
    Groups(Vec<ConsumerGroup>),
    Published(Message),
    Claimed(Option<Assignment>),
    Delivery(Delivery),
    Acked {
        delivery: Delivery,
        next: Option<Message>,
    },
    Message(MessageView),
    Messages(Vec<MessageView>),
}
pub trait Queue {
    fn execute(&mut self, command: Command) -> Result<Output>;
}

#[cfg(test)]
mod tests {
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
}
