use crate::*;

pub trait PublishMessage {
    fn publish(&mut self, request: Publish) -> Result<Message>;
}
