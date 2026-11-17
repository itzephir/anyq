use crate::CatalogRecords;
use anyq_plugin_api::InstalledPlugin;
use anyq_queue_api::Result;

pub trait CatalogStore {
    fn read(&mut self) -> Result<Vec<InstalledPlugin>>;

    fn change<T>(
        &mut self,
        operation: impl FnOnce(&mut dyn CatalogRecords) -> Result<T>,
    ) -> Result<T>;
}
