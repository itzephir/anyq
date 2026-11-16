use anyq_plugin_api::InstalledPlugin;
use anyq_queue_api::Result;

pub trait CatalogRecords {
    fn list(&self) -> Result<Vec<InstalledPlugin>>;
    fn save(&mut self, plugin: &InstalledPlugin) -> Result<()>;
}
