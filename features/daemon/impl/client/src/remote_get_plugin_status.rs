use crate::Channel;
use anyq_daemon_api::{QueueRequest, QueueResponse};
use anyq_queue_api::Result;

pub struct RemoteGetPluginStatus {
    channel: Channel,
}

impl RemoteGetPluginStatus {
    pub fn new(channel: Channel) -> Self {
        Self { channel }
    }
}

impl anyq_plugin_api::GetPluginStatus for RemoteGetPluginStatus {
    fn status(&mut self, name: &str) -> Result<anyq_plugin_api::PluginStatus> {
        match self
            .channel
            .request(QueueRequest::PluginStatus { name: name.into() })?
        {
            QueueResponse::PluginStatus(value) => value.try_into(),
            _ => Err(Channel::unexpected()),
        }
    }
}
