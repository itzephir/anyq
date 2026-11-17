use crate::CatalogStore;
use anyq_plugin_api::{DisableInstalledPlugin, InstalledPlugin, validate_catalog_name};
use anyq_queue_api::{Error, Result};

pub struct DisableInstalledPluginInteractor<S> {
    store: S,
}

impl<S> DisableInstalledPluginInteractor<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S: CatalogStore> DisableInstalledPlugin for DisableInstalledPluginInteractor<S> {
    fn disable(&mut self, name: &str) -> Result<InstalledPlugin> {
        validate_catalog_name(name)?;

        self.store.change(|records| {
            let mut plugin = records
                .list()?
                .into_iter()
                .find(|plugin| plugin.name == name)
                .ok_or_else(|| Error::NotFound(format!("plugin '{name}' is not installed")))?;
            plugin.enabled = false;
            plugin.revision = plugin
                .revision
                .checked_add(1)
                .filter(|revision| *revision <= i64::MAX as u64)
                .ok_or_else(|| Error::Invalid("plugin catalog revision exhausted".into()))?;
            records.save(&plugin)?;

            Ok(plugin)
        })
    }
}
