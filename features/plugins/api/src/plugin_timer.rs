#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginTimer {
    pub id: String,
    pub deadline: i64,
    pub origin_sequence: i64,
    pub payload: anyq_queue_api::Metadata,
}
