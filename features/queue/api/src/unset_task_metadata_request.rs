#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsetTaskMetadataRequest {
    pub task: String,
    pub key: String,
}
