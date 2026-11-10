use anyq_queue_api::Result;

pub trait ListPlugins {
    fn list(&mut self) -> Result<Vec<crate::PluginStatus>>;
}
