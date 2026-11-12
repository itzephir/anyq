#[derive(Debug)]
pub(crate) struct PendingDelivery {
    pub id: String,
    pub sequence: Option<i64>,
    pub timer: Option<(String, i64)>,
    pub origin_sequence: i64,
    pub sent_at: std::time::Instant,
}
