use crate::CatalogStore;
use anyq_plugin_api::{InstalledPlugin, ListInstalledPlugins};
use anyq_queue_api::Result;

pub struct ListInstalledPluginsInteractor<S> {
    store: S,
}

impl<S> ListInstalledPluginsInteractor<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S: CatalogStore> ListInstalledPlugins for ListInstalledPluginsInteractor<S> {
    fn list(&mut self) -> Result<Vec<InstalledPlugin>> {
        self.store.read()
    }
}
