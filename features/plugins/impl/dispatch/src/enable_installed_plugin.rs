use crate::CatalogStore;
use anyq_plugin_api::{
    EnableInstalledPlugin, InstalledPlugin, MAX_ENABLED_PLUGINS, validate_catalog_name,
};
use anyq_queue_api::{Error, Result};

pub struct EnableInstalledPluginInteractor<S> {
    store: S,
}

impl<S> EnableInstalledPluginInteractor<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S: CatalogStore> EnableInstalledPlugin for EnableInstalledPluginInteractor<S> {
    fn enable(&mut self, name: &str) -> Result<InstalledPlugin> {
        validate_catalog_name(name)?;

        self.store.change(|records| {
            let plugins = records.list()?;
            let mut plugin = plugins
                .iter()
                .find(|plugin| plugin.name == name)
                .cloned()
                .ok_or_else(|| Error::NotFound(format!("plugin '{name}' is not installed")))?;
            if !plugin.available {
                return Err(Error::Invalid(format!(
                    "plugin '{name}' binary is missing or not executable"
                )));
            }
            if !plugin.enabled
                && plugins.iter().filter(|plugin| plugin.enabled).count() >= MAX_ENABLED_PLUGINS
            {
                return Err(Error::Conflict("enabled plugin limit reached (16)".into()));
            }

            plugin.enabled = true;
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
