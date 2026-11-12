#[derive(Debug, Default, Clone)]
pub struct ListManagedWorkRequest {
    pub owner: Option<String>,
    pub task: Option<String>,
    pub after: Option<String>,
    pub limit: Option<usize>,
    pub states: Vec<crate::ManagedWorkState>,
}
