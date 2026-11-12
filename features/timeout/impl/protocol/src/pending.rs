use anyq_timeout_api::Deadline;

pub(crate) struct Pending {
    pub delivery_id: String,
    pub plan: Deadline,
}
