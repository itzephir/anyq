use crate::ports::Store;
use anyq_config_api::{Config, Result, SetConfigRequest, SetConfigValue};

pub struct SetConfigValueInteractor<S> {
    store: S,
}

impl<S> SetConfigValueInteractor<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S: Store> SetConfigValue for SetConfigValueInteractor<S> {
    fn set(&mut self, request: SetConfigRequest) -> Result<Config> {
        self.store.change(|config| {
            config.set(request.key, request.value)?;

            Ok(config.clone())
        })
    }
}
