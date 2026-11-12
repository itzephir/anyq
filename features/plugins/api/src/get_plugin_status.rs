use anyq_queue_api::Result;

pub trait GetPluginStatus {
    fn status(&mut self, name: &str) -> Result<crate::PluginStatus>;
}
