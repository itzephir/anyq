use anyq_queue_api::State;

#[derive(Default)]
pub struct DeliveryFilter {
    pub topic: Option<String>,
    pub group: Option<String>,
    pub message_id: Option<i64>,
    pub state: Option<State>,
    pub ready_at: Option<i64>,
    pub expired_at: Option<i64>,
    pub limit: Option<u32>,
}
