#[derive(Debug, Clone)]
pub struct CompleteTaskOnceRequest {
    pub owner: String,
    pub action_id: String,
    pub task: String,
    pub force: bool,
}
