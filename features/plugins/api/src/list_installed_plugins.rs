use crate::InstalledPlugin;
use anyq_queue_api::Result;

pub trait ListInstalledPlugins {
    fn list(&mut self) -> Result<Vec<InstalledPlugin>>;
}
