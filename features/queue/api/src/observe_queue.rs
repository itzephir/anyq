use crate::{ObserveQueueRequest, QueueSnapshot, Result};

pub trait ObserveQueue {
    fn observe(&mut self, request: ObserveQueueRequest) -> Result<QueueSnapshot>;
}
