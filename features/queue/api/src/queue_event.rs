use crate::EventSnapshot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueueEvent {
    pub database_id: String,
    pub sequence: i64,
    pub version: u32,
    pub snapshot: EventSnapshot,
}

impl QueueEvent {
    pub fn id(&self) -> String {
        format!("{}:{}", self.database_id, self.sequence)
    }
}
