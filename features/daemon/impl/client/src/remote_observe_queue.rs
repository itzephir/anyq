use crate::Channel;
use anyq_daemon_api::{QueueRequest, QueueResponse};
use anyq_queue_api::{ObserveQueue, ObserveQueueRequest, QueueSnapshot, Result};

pub struct RemoteObserveQueue {
    channel: Channel,
    database_id: String,
}

impl RemoteObserveQueue {
    pub fn new(channel: Channel, database_id: String) -> Self {
        Self {
            channel,
            database_id,
        }
    }
}

impl ObserveQueue for RemoteObserveQueue {
    fn observe(&mut self, request: ObserveQueueRequest) -> Result<QueueSnapshot> {
        request.validate()?;

        match self.channel.request(QueueRequest::ObserveQueue {
            max_entities: request.max_entities,
            max_text_bytes: request.max_text_bytes,
        })? {
            QueueResponse::Observation {
                database_id,
                snapshot,
            } if database_id == self.database_id => snapshot.try_into(),
            _ => Err(Channel::unexpected()),
        }
    }
}
