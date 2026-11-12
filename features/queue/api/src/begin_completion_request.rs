#[derive(Debug, Clone)]
pub struct BeginCompletionRequest {
    pub task: String,
    pub force: bool,
}
