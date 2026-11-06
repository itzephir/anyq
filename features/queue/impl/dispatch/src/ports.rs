//! Consumer-owned transactional persistence and nondeterminism ports.
use anyq_queue_api::*;

#[derive(Default)]
pub struct DeliveryFilter {
    pub topic: Option<String>,
    pub group: Option<String>,
    pub message_id: Option<i64>,
    pub state: Option<State>,
    pub ready_at: Option<i64>,
    pub expired_at: Option<i64>,
    pub limit: Option<u32>,
}
pub trait Records {
    fn topics(&mut self) -> Result<Vec<Topic>>;
    fn topic(&mut self, name: &str) -> Result<Option<Topic>>;
    fn put_topic(&mut self, topic: &Topic) -> Result<()>;
    fn groups(&mut self, topic: &str) -> Result<Vec<ConsumerGroup>>;
    fn insert_group(&mut self, group: &ConsumerGroup) -> Result<()>;
    /// Insert pending deliveries from retained messages with their original publication times.
    fn seed_group(&mut self, group: &ConsumerGroup) -> Result<()>;
    fn message(&mut self, id: i64) -> Result<Option<Message>>;
    fn message_by_key(&mut self, topic: &str, key: &str) -> Result<Option<Message>>;
    fn insert_message(&mut self, request: &Publish, now: i64) -> Result<Message>;
    fn messages(&mut self, topic: Option<&str>, after: i64, limit: u32) -> Result<Vec<Message>>;
    fn deliveries(&mut self, filter: &DeliveryFilter) -> Result<Vec<Delivery>>;
    fn count_deliveries(&mut self, filter: &DeliveryFilter) -> Result<u64>;
    fn put_delivery(&mut self, delivery: &Delivery) -> Result<()>;
}
pub trait Store {
    /// A failed operation MUST roll back every write, including publication of follow-up messages.
    fn transaction<T>(
        &mut self,
        operation: impl FnOnce(&mut dyn Records) -> Result<T>,
    ) -> Result<T>;
}
pub trait Clock {
    fn now_ms(&self) -> i64;
}
pub trait Tokens {
    fn next(&mut self) -> String;
}
