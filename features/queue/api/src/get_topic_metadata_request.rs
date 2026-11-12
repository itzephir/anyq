#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetTopicMetadataRequest {
    pub topic: String,
    pub key: Option<String>,
}
