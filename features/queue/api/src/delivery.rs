use crate::{Error, Message, Result, State, deadline, validate_name};

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
