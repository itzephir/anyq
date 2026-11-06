use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Completion {
    pub delivery: Delivery,
    pub next: Option<Message>,
}
