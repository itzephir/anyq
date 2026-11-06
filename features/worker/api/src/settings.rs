use anyq_queue_api::{Claim, Publish, Result};

#[derive(Clone, Debug)]
pub struct Settings {
    pub claim: Claim,
    pub retry_delay_ms: i64,
    pub idle_ms: u64,
    pub next: Option<Publish>,
}

impl Settings {
    pub fn validate(&self) -> Result<()> {
        use anyq_queue_api::{Error, deadline, validate_name};

        validate_name(&self.claim.topic)?;
        validate_name(&self.claim.group)?;
        validate_name(&self.claim.consumer)?;
        deadline(0, self.claim.lease_ms)?;

        if self.claim.lease_ms < 300
            || !(0..=86_400_000).contains(&self.retry_delay_ms)
            || !(1..=60_000).contains(&self.idle_ms)
        {
            return Err(Error::Invalid(
                "worker needs lease >= 300ms, retry delay 0..86400000ms and poll 1..60000ms".into(),
            ));
        }

        if let Some(next) = &self.next {
            next.validate()?;
        }

        Ok(())
    }
}
