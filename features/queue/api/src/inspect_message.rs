use crate::*;

pub trait InspectMessage {
    fn inspect(&mut self, message_id: i64) -> Result<MessageView>;
}
