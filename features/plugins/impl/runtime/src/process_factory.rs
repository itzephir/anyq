use crate::PluginProcess;
use anyq_plugin_api::PluginRegistration;
use anyq_queue_api::Result;

pub trait ProcessFactory {
    fn spawn(&self, registration: &PluginRegistration) -> Result<Box<dyn PluginProcess>>;
}
