use crate::{Config, ConfigKey, Result};

pub trait UnsetConfigValue {
    fn unset(&mut self, key: ConfigKey) -> Result<Config>;
}
