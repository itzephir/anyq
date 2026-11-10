use crate::{Error, Metadata, MetadataValue, Result};

/// Incremental structural limits shared by values and streaming input adapters.
#[derive(Debug, Default)]
pub struct MetadataBudget {
    bytes: usize,
}

impl MetadataBudget {
    fn charge(&mut self, bytes: usize) -> Result<()> {
        self.bytes = self
            .bytes
            .checked_add(bytes)
            .filter(|bytes| *bytes <= Metadata::MAX_BYTES)
            .ok_or_else(|| Error::Invalid("metadata exceeds 1 MiB".into()))?;

        Ok(())
    }

    pub fn add_value(&mut self, depth: usize) -> Result<()> {
        if depth > Metadata::MAX_DEPTH {
            return Err(Error::Invalid("metadata exceeds 64 levels".into()));
        }

        self.charge(Metadata::VALUE_BYTES)
    }

    pub fn add_text(&mut self, text: &str) -> Result<()> {
        self.charge(text.len())
    }

    pub fn include(&mut self, value: &MetadataValue, depth: usize) -> Result<()> {
        let mut pending = vec![(value, depth)];

        while let Some((value, depth)) = pending.pop() {
            self.add_value(depth)?;

            match value {
                MetadataValue::String(text) => self.add_text(text)?,
                MetadataValue::Number(number) => self.add_text(number.as_str())?,
                MetadataValue::Array(items) => {
                    if items.len() > Metadata::MAX_BYTES / Metadata::VALUE_BYTES {
                        return Err(Error::Invalid("metadata exceeds 1 MiB".into()));
                    }
                    pending.extend(items.iter().map(|item| (item, depth + 1)));
                }
                MetadataValue::Object(items) => {
                    if items.len() > Metadata::MAX_BYTES / Metadata::VALUE_BYTES {
                        return Err(Error::Invalid("metadata exceeds 1 MiB".into()));
                    }
                    for (key, item) in items {
                        self.add_text(key)?;
                        pending.push((item, depth + 1));
                    }
                }
                MetadataValue::Null | MetadataValue::Bool(_) => {}
            }
        }

        Ok(())
    }
}
