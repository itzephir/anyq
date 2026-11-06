#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub id: i64,
    pub topic: String,
    pub payload: String,
    pub key: Option<String>,
    pub max_attempts: u32,
    pub created_at: i64,
}
