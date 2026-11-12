use crate::Store;
use anyq_plugin_api::*;
use anyq_queue_api::{Error, Result};

pub struct DisablePluginInteractor<S> {
    store: S,
}

impl<S> DisablePluginInteractor<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S: Store> DisablePlugin for DisablePluginInteractor<S> {
    fn disable(&mut self, name: &str) -> Result<PluginStatus> {
        validate_plugin_name(name)?;

        self.store.transaction(|records| {
            let mut registration = records
                .registration(name)?
                .ok_or_else(|| Error::NotFound("plugin is not registered".into()))?;
            registration.enabled = false;
            records.save(&registration)?;

            crate::read_status::read_status(records, name)
        })
    }
}
