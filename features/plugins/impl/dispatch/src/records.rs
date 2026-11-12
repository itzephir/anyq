use anyq_plugin_api::{PluginRegistration, PluginTimer};
use anyq_queue_api::Result;

pub trait Records {
    fn prune_events_before(&mut self, sequence: i64) -> Result<()>;

    fn registration(&mut self, name: &str) -> Result<Option<PluginRegistration>>;

    fn registrations(&mut self) -> Result<Vec<PluginRegistration>>;

    fn save(&mut self, registration: &PluginRegistration) -> Result<()>;

    fn count_managed(
        &mut self,
        owner: &str,
        states: &[anyq_queue_api::ManagedWorkState],
    ) -> Result<u64>;

    fn watermark(&mut self) -> Result<i64>;

    fn timers(&mut self, plugin: &str) -> Result<Vec<PluginTimer>>;

    fn put_timer(&mut self, plugin: &str, timer: &PluginTimer) -> Result<()>;

    fn remove_timer(&mut self, plugin: &str, id: &str) -> Result<()>;
}
