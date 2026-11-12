use crate::Store;
use anyq_plugin_api::*;
use anyq_queue_api::Result;

pub struct GetPluginStatusInteractor<S> {
    store: S,
}

impl<S> GetPluginStatusInteractor<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S: Store> GetPluginStatus for GetPluginStatusInteractor<S> {
    fn status(&mut self, name: &str) -> Result<PluginStatus> {
        validate_plugin_name(name)?;

        self.store
            .transaction(|records| crate::read_status::read_status(records, name))
    }
}
