#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginStatus {
    pub state: anyq_queue_api::Metadata,
    pub name: String,
    pub enabled: bool,
    pub running: bool,
    pub cursor: i64,
    pub lag: i64,
    pub failures: u32,
    pub last_failure: Option<String>,
    pub active_timers: u64,
    pub unresolved_actions: u64,
}
