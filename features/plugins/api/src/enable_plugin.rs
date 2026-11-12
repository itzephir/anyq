use anyq_queue_api::Result;

pub trait EnablePlugin {
    fn enable(&mut self, request: crate::EnablePluginRequest) -> Result<crate::PluginStatus>;
}
