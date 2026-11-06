use crate::{Delivery, Message};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageView {
    pub message: Message,
    pub deliveries: Vec<Delivery>,
}
