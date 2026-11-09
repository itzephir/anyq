use crate::{ConfigKey, ConfigValue, Result};

pub trait GetConfigValue {
    fn get(&mut self, key: ConfigKey) -> Result<Option<ConfigValue>>;
}
