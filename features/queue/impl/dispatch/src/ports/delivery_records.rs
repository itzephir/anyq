use super::DeliveryFilter;
use anyq_queue_api::*;

pub trait DeliveryRecords {
    fn seed_group(&mut self, group: &ConsumerGroup) -> Result<()>;

    fn deliveries(&mut self, filter: &DeliveryFilter) -> Result<Vec<Delivery>>;

    fn count_deliveries(&mut self, filter: &DeliveryFilter) -> Result<u64>;

    fn put_delivery(&mut self, delivery: &Delivery) -> Result<()>;
}
