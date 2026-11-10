use crate::Channel;
use anyq_daemon_api::{QueueRequest, QueueResponse};
use anyq_queue_api::Result;

pub struct RemoteListPlugins {
    channel: Channel,
}

impl RemoteListPlugins {
    pub fn new(channel: Channel) -> Self {
        Self { channel }
    }
}

impl anyq_plugin_api::ListPlugins for RemoteListPlugins {
    fn list(&mut self) -> Result<Vec<anyq_plugin_api::PluginStatus>> {
        match self.channel.request(QueueRequest::PluginList)? {
            QueueResponse::Plugins(value) => value.into_iter().map(TryInto::try_into).collect(),
            _ => Err(Channel::unexpected()),
        }
    }
}
