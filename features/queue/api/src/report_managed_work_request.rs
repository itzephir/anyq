#[derive(Debug, Clone)]
pub struct ReportManagedWorkRequest {
    pub owner: String,
    pub action_id: String,
    pub state: crate::ManagedWorkState,
}
