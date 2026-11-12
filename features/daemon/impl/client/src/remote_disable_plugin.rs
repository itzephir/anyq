use crate::Channel;
use anyq_daemon_api::{QueueRequest, QueueResponse};
use anyq_queue_api::Result;

pub struct RemoteDisablePlugin {
    channel: Channel,
}

impl RemoteDisablePlugin {
    pub fn new(channel: Channel) -> Self {
        Self { channel }
    }
}

impl anyq_plugin_api::DisablePlugin for RemoteDisablePlugin {
    fn disable(&mut self, name: &str) -> Result<anyq_plugin_api::PluginStatus> {
        match self
            .channel
            .request(QueueRequest::PluginDisable { name: name.into() })?
        {
            QueueResponse::PluginStatus(value) => value.try_into(),
            _ => Err(Channel::unexpected()),
        }
    }
}
