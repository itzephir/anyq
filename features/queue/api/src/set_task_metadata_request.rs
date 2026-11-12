#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetTaskMetadataRequest {
    pub task: String,
    pub key: String,
    pub value: crate::MetadataValue,
}
