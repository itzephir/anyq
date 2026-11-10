#[derive(Debug, Clone)]
pub struct ResolveManagedWorkRequest {
    pub owner: String,
    pub action_id: String,
    pub confirmed_cleanup: bool,
}
