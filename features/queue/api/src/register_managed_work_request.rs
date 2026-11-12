#[derive(Debug, Clone)]
pub struct RegisterManagedWorkRequest {
    pub details: crate::Metadata,

    pub owner: String,
    pub action_id: String,
    pub task: String,
}
