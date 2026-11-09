use crate::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigKey {
    TaskAutocreate,
    TopicAutocreate,
    DbPath,
}

impl ConfigKey {
    pub const ALL: [Self; 3] = [Self::TaskAutocreate, Self::TopicAutocreate, Self::DbPath];

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "task.autocreate" => Ok(Self::TaskAutocreate),
            "topic.autocreate" => Ok(Self::TopicAutocreate),
            "db.path" => Ok(Self::DbPath),
            _ => Err(Error::Invalid(format!(
                "unknown configuration key '{value}'"
            ))),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::TaskAutocreate => "task.autocreate",
            Self::TopicAutocreate => "topic.autocreate",
            Self::DbPath => "db.path",
        }
    }

    pub fn require_section(name: &str) -> Result<()> {
        if Self::ALL.iter().any(|key| {
            key.as_str()
                .strip_prefix(name)
                .is_some_and(|suffix| suffix.starts_with('.'))
        }) {
            return Ok(());
        }

        Err(Error::Invalid(format!(
            "unsupported configuration section '{name}'"
        )))
    }
}
