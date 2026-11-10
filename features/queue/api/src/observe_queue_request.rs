use crate::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObserveQueueRequest {
    pub max_entities: u32,
    pub max_text_bytes: u64,
}

impl Default for ObserveQueueRequest {
    fn default() -> Self {
        Self {
            max_entities: 10_000,
            max_text_bytes: 1024 * 1024,
        }
    }
}

impl ObserveQueueRequest {
    pub fn validate(self) -> Result<()> {
        let ceiling = Self::default();

        if self.max_entities == 0
            || self.max_entities > ceiling.max_entities
            || self.max_text_bytes == 0
            || self.max_text_bytes > ceiling.max_text_bytes
        {
            return Err(Error::Invalid(
                "unsupported queue observation limits".into(),
            ));
        }

        Ok(())
    }

    pub fn require_size(self, entities: u64, text_bytes: u64) -> Result<()> {
        self.validate()?;

        if entities > u64::from(self.max_entities) || text_bytes > self.max_text_bytes {
            return Err(Error::Invalid(
                "queue snapshot exceeds observation size limit".into(),
            ));
        }

        Ok(())
    }
}
