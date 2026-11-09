use anyq_queue_api::TaskState;

/// Exact query constraints selected by the use case.
pub struct TaskFilter {
    pub topic: Option<String>,
    pub state: TaskState,
    pub before_sequence: Option<i64>,
}
