use crate::{ConfigKey, ConfigValue, Error, Result};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Config {
    task_autocreate: Option<bool>,
    topic_autocreate: Option<bool>,
    db_path: Option<String>,
}

impl Config {
    pub fn task_autocreate(&self) -> bool {
        self.task_autocreate.unwrap_or(false)
    }

    pub fn topic_autocreate(&self) -> bool {
        self.topic_autocreate.unwrap_or(false)
    }

    pub fn db_path(&self) -> Option<&str> {
        self.db_path.as_deref()
    }

    pub fn get(&self, key: ConfigKey) -> Option<ConfigValue> {
        match key {
            ConfigKey::TaskAutocreate => Some(ConfigValue::Boolean(self.task_autocreate())),
            ConfigKey::TopicAutocreate => Some(ConfigValue::Boolean(self.topic_autocreate())),
            ConfigKey::DbPath => self.db_path.clone().map(ConfigValue::Text),
        }
    }

    pub fn set(&mut self, key: ConfigKey, value: ConfigValue) -> Result<()> {
        match (key, value) {
            (ConfigKey::TaskAutocreate, ConfigValue::Boolean(value)) => {
                self.task_autocreate = Some(value)
            }
            (ConfigKey::TopicAutocreate, ConfigValue::Boolean(value)) => {
                self.topic_autocreate = Some(value)
            }
            (ConfigKey::DbPath, ConfigValue::Text(value)) if !value.is_empty() => {
                self.db_path = Some(value)
            }
            _ => {
                return Err(Error::Invalid(format!(
                    "invalid value for {}",
                    key.as_str()
                )));
            }
        }

        Ok(())
    }

    pub fn unset(&mut self, key: ConfigKey) {
        match key {
            ConfigKey::TaskAutocreate => self.task_autocreate = None,
            ConfigKey::TopicAutocreate => self.topic_autocreate = None,
            ConfigKey::DbPath => self.db_path = None,
        }
    }

    pub fn overrides(&self) -> Vec<(ConfigKey, ConfigValue)> {
        let mut values = Vec::new();

        if let Some(value) = self.task_autocreate {
            values.push((ConfigKey::TaskAutocreate, ConfigValue::Boolean(value)));
        }

        if let Some(value) = self.topic_autocreate {
            values.push((ConfigKey::TopicAutocreate, ConfigValue::Boolean(value)));
        }

        if let Some(value) = &self.db_path {
            values.push((ConfigKey::DbPath, ConfigValue::Text(value.clone())));
        }

        values
    }
}
