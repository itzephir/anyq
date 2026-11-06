use anyq_queue_api::{GetTopic, Result, Topic};
pub struct TopicStub;
impl GetTopic for TopicStub {
    fn get(&mut self, name: &str) -> Result<Topic> {
        Topic::new(name.into(), 1)
    }
}
