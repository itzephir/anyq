use crate::*;

pub trait SetTopicCapacity {
    fn set_capacity(&mut self, topic: Topic) -> Result<Topic>;
}
