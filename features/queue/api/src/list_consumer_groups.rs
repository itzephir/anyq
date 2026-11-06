use crate::*;

pub trait ListConsumerGroups {
    fn list(&mut self, topic: &str) -> Result<Vec<ConsumerGroup>>;
}
