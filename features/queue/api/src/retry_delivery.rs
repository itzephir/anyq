use crate::*;

pub trait RetryDelivery {
    fn retry(&mut self, message_id: i64, group: String) -> Result<Delivery>;
}
