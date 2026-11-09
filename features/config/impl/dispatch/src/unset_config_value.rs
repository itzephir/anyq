use crate::ports::Store;
use anyq_config_api::{Config, ConfigKey, Result, UnsetConfigValue};

pub struct UnsetConfigValueInteractor<S> {
    store: S,
}

impl<S> UnsetConfigValueInteractor<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S: Store> UnsetConfigValue for UnsetConfigValueInteractor<S> {
    fn unset(&mut self, key: ConfigKey) -> Result<Config> {
        self.store.change(|config| {
            config.unset(key);

            Ok(config.clone())
        })
    }
}
