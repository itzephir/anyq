use crate::Store;
use anyq_plugin_api::*;
use anyq_queue_api::Result;

pub struct ListPluginsInteractor<S> {
    store: S,
}

impl<S> ListPluginsInteractor<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S: Store> ListPlugins for ListPluginsInteractor<S> {
    fn list(&mut self) -> Result<Vec<PluginStatus>> {
        self.store.transaction(|records| {
            records
                .registrations()?
                .iter()
                .map(|registration| crate::read_status::read_status(records, &registration.name))
                .collect()
        })
    }
}
