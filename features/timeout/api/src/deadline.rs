use anyq_queue_api::{Error, Metadata, MetadataNumber, MetadataValue, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deadline {
    pub event_id: String,
    pub task: String,
    pub deadline: i64,
    pub source: String,
    pub value: String,
}

impl Deadline {
    pub fn metadata(&self) -> Result<Metadata> {
        let mut value = Metadata::default();
        for (key, text) in [
            ("event_id", &self.event_id),
            ("task", &self.task),
            ("source", &self.source),
            ("value", &self.value),
        ] {
            value.set(key, MetadataValue::String(text.clone()))?;
        }
        value.set(
            "deadline",
            MetadataValue::Number(MetadataNumber::parse(&self.deadline.to_string())?),
        )?;

        Ok(value)
    }

    pub fn restore(value: Metadata) -> Result<Self> {
        fn text(value: &Metadata, key: &str) -> Result<String> {
            match value.get(Some(key))? {
                MetadataValue::String(text) => Ok(text.clone()),
                _ => Err(Error::Invalid("invalid saved timeout identity".into())),
            }
        }
        let deadline = match value.get(Some("deadline"))? {
            MetadataValue::Number(number) => number.as_str().parse().ok(),
            _ => None,
        }
        .ok_or_else(|| Error::Invalid("invalid saved timeout deadline".into()))?;

        Ok(Self {
            event_id: text(&value, "event_id")?,
            task: text(&value, "task")?,
            deadline,
            source: text(&value, "source")?,
            value: text(&value, "value")?,
        })
    }
}
