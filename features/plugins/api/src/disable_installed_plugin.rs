use crate::InstalledPlugin;
use anyq_queue_api::Result;

pub trait DisableInstalledPlugin {
    fn disable(&mut self, name: &str) -> Result<InstalledPlugin>;
}
