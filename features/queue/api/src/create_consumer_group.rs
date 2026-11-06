use crate::*;

pub trait CreateConsumerGroup {
    fn create(&mut self, group: ConsumerGroup) -> Result<ConsumerGroup>;
}
