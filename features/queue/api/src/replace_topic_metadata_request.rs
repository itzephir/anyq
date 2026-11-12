#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplaceTopicMetadataRequest {
    pub topic: String,
    pub metadata: crate::Metadata,
}
