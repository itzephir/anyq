#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lease {
    pub message_id: i64,
    pub group: String,
    pub token: String,
}
