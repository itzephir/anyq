use anyq_queue_api::Result;

pub trait DisablePlugin {
    fn disable(&mut self, name: &str) -> Result<crate::PluginStatus>;
}
