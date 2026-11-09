use crate::ports::Store;
use anyq_config_api::{Config, GetConfig, Result};

pub struct GetConfigInteractor<S> {
    store: S,
}

impl<S> GetConfigInteractor<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S: Store> GetConfig for GetConfigInteractor<S> {
    fn get(&mut self) -> Result<Config> {
        self.store.read()
    }
}
