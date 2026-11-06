use anyq_queue_api::*;

pub trait TopicRecords {
    fn topics(&mut self) -> Result<Vec<Topic>>;
    fn topic(&mut self, name: &str) -> Result<Option<Topic>>;
    fn put_topic(&mut self, topic: &Topic) -> Result<()>;
}
