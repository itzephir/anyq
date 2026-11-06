use anyq_queue_api::*;

pub trait GroupRecords {
    fn groups(&mut self, topic: &str) -> Result<Vec<ConsumerGroup>>;
    fn insert_group(&mut self, group: &ConsumerGroup) -> Result<()>;
}
