#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumerGroup {
    pub topic: String,
    pub name: String,
}
