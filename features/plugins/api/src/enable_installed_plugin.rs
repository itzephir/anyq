use crate::InstalledPlugin;
use anyq_queue_api::Result;

pub trait EnableInstalledPlugin {
    fn enable(&mut self, name: &str) -> Result<InstalledPlugin>;
}
