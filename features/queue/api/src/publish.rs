use crate::{Error, Result, validate_name};

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
