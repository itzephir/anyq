use crate::ports::Store;
use anyq_config_api::{ConfigKey, ConfigValue, GetConfigValue, Result};

pub struct GetConfigValueInteractor<S> {
    store: S,
}

impl<S> GetConfigValueInteractor<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S: Store> GetConfigValue for GetConfigValueInteractor<S> {
    fn get(&mut self, key: ConfigKey) -> Result<Option<ConfigValue>> {
        Ok(self.store.read()?.get(key))
    }
}
