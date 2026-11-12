use anyq_queue_api::{ObserveQueueRequest, QueueSnapshot, Result, TaskState};

pub trait ObservationStore {
    fn snapshot(
        &mut self,
        states: &[TaskState; 2],
        request: ObserveQueueRequest,
    ) -> Result<QueueSnapshot>;
}
