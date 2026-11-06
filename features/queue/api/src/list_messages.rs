use crate::*;

pub trait ListMessages {
    fn list(&mut self, query: MessageQuery) -> Result<Vec<MessageView>>;
}
