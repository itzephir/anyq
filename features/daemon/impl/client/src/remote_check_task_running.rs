use crate::Channel;
use anyq_daemon_api::{QueueRequest, QueueResponse};
use anyq_queue_api::{CheckTaskRunning, Result};

pub struct RemoteCheckTaskRunning {
    channel: Channel,
}

impl RemoteCheckTaskRunning {
    pub fn new(channel: Channel) -> Self {
        Self { channel }
    }
}

impl CheckTaskRunning for RemoteCheckTaskRunning {
    fn check_running(&mut self, name: &str) -> Result<bool> {
        match self
            .channel
            .request(QueueRequest::CheckTaskRunning { name: name.into() })?
        {
            QueueResponse::Running(value) => Ok(value),
            _ => Err(Channel::unexpected()),
        }
    }
}
