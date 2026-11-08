use crate::{Result, TopicQueue};

pub trait GetTopicQueue {
    fn get(&mut self, name: &str) -> Result<TopicQueue>;
}
