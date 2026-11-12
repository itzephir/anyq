#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginRegistration {
    pub name: String,
    pub executable: String,
    pub arguments: Vec<String>,
    pub enabled: bool,
    pub running: bool,
    pub cursor: i64,
    pub failures: u32,
    pub retry_at: i64,
    pub last_failure: Option<String>,
    pub subscriptions: Vec<String>,
    pub capabilities: Vec<String>,
    pub state: anyq_queue_api::Metadata,
}
