use anyq_queue_api::*;

pub trait MessageRecords {
    fn message(&mut self, id: i64) -> Result<Option<Message>>;

    fn message_by_key(&mut self, topic: &str, key: &str) -> Result<Option<Message>>;

    fn insert_message(&mut self, request: &Publish, now: i64) -> Result<Message>;

    fn messages(&mut self, topic: Option<&str>, after: i64, limit: u32) -> Result<Vec<Message>>;
}
