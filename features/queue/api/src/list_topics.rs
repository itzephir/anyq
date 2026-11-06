use crate::*;

pub trait ListTopics {
    fn list(&mut self) -> Result<Vec<Topic>>;
}
