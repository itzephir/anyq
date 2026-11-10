use crate::Channel;
use anyq_daemon_api::{QueueRequest, QueueResponse};
use anyq_queue_api::{Result, StartTask, StartTaskOutcome};

pub struct RemoteStartTask {
    channel: Channel,
}

impl RemoteStartTask {
    pub fn new(channel: Channel) -> Self {
        Self { channel }
    }
}

impl StartTask for RemoteStartTask {
    fn start(&mut self, name: &str) -> Result<StartTaskOutcome> {
        match self
            .channel
            .request(QueueRequest::StartTask { name: name.into() })?
        {
            QueueResponse::Started(value) => match value.as_str() {
                "started" => Ok(StartTaskOutcome::Started),
                "already-running" => Ok(StartTaskOutcome::AlreadyRunning),
                "busy" => Ok(StartTaskOutcome::Busy),
                _ => Err(Channel::unexpected()),
            },
            _ => Err(Channel::unexpected()),
        }
    }
}
