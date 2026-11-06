use crate::{Delivery, Lease, Message};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assignment {
    pub message: Message,
    pub delivery: Delivery,
}

impl Assignment {
    pub fn lease(&self) -> Lease {
        Lease {
            message_id: self.message.id,
            group: self.delivery.group.clone(),
            token: self.delivery.token.clone().unwrap_or_default(),
        }
    }
}
