use crate::{ConfigKey, Error, Result};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigValue {
    Boolean(bool),
    Text(String),
}

impl ConfigValue {
    pub fn parse(key: ConfigKey, input: &str) -> Result<Self> {
        match key {
            ConfigKey::TaskAutocreate | ConfigKey::TopicAutocreate => match input {
                "true" => Ok(Self::Boolean(true)),
                "false" => Ok(Self::Boolean(false)),
                _ => Err(Error::Invalid(format!(
                    "{} requires true or false",
                    key.as_str()
                ))),
            },
            ConfigKey::DbPath => Ok(Self::Text(input.into())),
        }
    }
}

impl Display for ConfigValue {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Boolean(value) => write!(formatter, "{value}"),
            Self::Text(value) => formatter.write_str(value),
        }
    }
}
