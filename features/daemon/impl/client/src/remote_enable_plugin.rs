use crate::Channel;
use anyq_daemon_api::{QueueRequest, QueueResponse};
use anyq_queue_api::Result;

pub struct RemoteEnablePlugin {
    channel: Channel,
}

impl RemoteEnablePlugin {
    pub fn new(channel: Channel) -> Self {
        Self { channel }
    }
}

impl anyq_plugin_api::EnablePlugin for RemoteEnablePlugin {
    fn enable(
        &mut self,
        request: anyq_plugin_api::EnablePluginRequest,
    ) -> Result<anyq_plugin_api::PluginStatus> {
        match self.channel.request(QueueRequest::PluginEnable {
            name: request.name,
            executable: request.executable,
            arguments: request.arguments,
        })? {
            QueueResponse::PluginStatus(value) => Ok(value.into()),
            _ => Err(Channel::unexpected()),
        }
    }
}
