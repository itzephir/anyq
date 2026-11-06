use crate::*;

pub trait GetTopic {
    fn get(&mut self, name: &str) -> Result<Topic>;
}
