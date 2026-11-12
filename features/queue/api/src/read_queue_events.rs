use crate::{QueueEvent, Result};

pub trait ReadQueueEvents {
    fn after(&mut self, sequence: i64, limit: usize) -> Result<Vec<QueueEvent>>;

    fn watermark(&mut self) -> Result<i64>;
}
