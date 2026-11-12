#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcknowledgeRequest {
    pub plugin: String,
    pub sequence: Option<i64>,
    pub timer: Option<(String, i64)>,
    pub state: Option<anyq_queue_api::Metadata>,
    pub timers: Vec<crate::TimerUpdate>,
}
