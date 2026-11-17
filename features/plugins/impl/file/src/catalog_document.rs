use crate::enablement_dto::EnablementDto;
use anyq_plugin_api::{MAX_ENABLED_PLUGINS, validate_catalog_name};
use anyq_queue_api::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CatalogDocument {
    pub version: u32,
    pub plugins: BTreeMap<String, EnablementDto>,
}

impl Default for CatalogDocument {
    fn default() -> Self {
        Self {
            version: 1,
            plugins: BTreeMap::new(),
        }
    }
}

impl CatalogDocument {
    pub fn validate(&self) -> Result<()> {
        if self.version != 1 || self.plugins.len() > 4096 {
            return Err(Error::Invalid(
                "unsupported or oversized plugin catalog".into(),
            ));
        }
        if self
            .plugins
            .values()
            .filter(|plugin| plugin.enabled)
            .count()
            > MAX_ENABLED_PLUGINS
        {
            return Err(Error::Invalid("enabled plugin limit exceeded (16)".into()));
        }
        for (name, plugin) in &self.plugins {
            validate_catalog_name(name)?;
            if plugin.revision > i64::MAX as u64 {
                return Err(Error::Invalid(
                    "plugin catalog revision is too large".into(),
                ));
            }
        }

        Ok(())
    }
}
