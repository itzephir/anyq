#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Claim {
    pub topic: String,
    pub group: String,
    pub consumer: String,
    pub lease_ms: i64,
}
