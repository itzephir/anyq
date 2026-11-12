#[derive(Debug, Clone)]
pub struct ListManagedWorkRequest {
    pub owner: Option<String>,
    pub task: Option<String>,
}
