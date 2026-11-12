#[derive(Debug, Clone)]
pub struct ReportManagedWorkRequest {
    pub details: Option<crate::Metadata>,

    pub owner: String,
    pub action_id: String,
    pub state: crate::ManagedWorkState,
}
