#[derive(Debug, Clone)]
pub struct RegisterManagedWorkRequest {
    pub owner: String,
    pub action_id: String,
    pub task: String,
}
