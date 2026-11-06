use crate::*;

pub trait CreateTopic {
    fn create(&mut self, topic: Topic) -> Result<Topic>;
}
