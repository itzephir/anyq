#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplaceTaskMetadataRequest {
    pub task: String,
    pub metadata: crate::Metadata,
}
