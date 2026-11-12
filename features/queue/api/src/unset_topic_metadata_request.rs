#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsetTopicMetadataRequest {
    pub topic: String,
    pub key: String,
}
