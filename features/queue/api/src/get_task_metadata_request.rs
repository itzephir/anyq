#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetTaskMetadataRequest {
    pub task: String,
    pub key: Option<String>,
}
