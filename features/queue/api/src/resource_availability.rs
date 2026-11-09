use crate::Topic;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceAvailability {
    pub topic: Topic,
    pub running: u64,
    pub queued_before: u64,
}

impl ResourceAvailability {
    pub fn permits_start(&self) -> bool {
        u64::from(self.topic.capacity)
            .checked_sub(self.running)
            .is_some_and(|free| self.queued_before < free)
    }
}
