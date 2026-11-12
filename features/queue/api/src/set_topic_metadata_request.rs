#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetTopicMetadataRequest {
    pub topic: String,
    pub key: String,
    pub value: crate::MetadataValue,
}
