use crate::ports::ObservationStore;
use anyq_queue_api::{ObserveQueue, ObserveQueueRequest, QueueSnapshot, Result, TopicQueue};

pub struct ObserveQueueInteractor<S> {
    store: S,
}

impl<S> ObserveQueueInteractor<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S: ObservationStore> ObserveQueue for ObserveQueueInteractor<S> {
    fn observe(&mut self, request: ObserveQueueRequest) -> Result<QueueSnapshot> {
        request.validate()?;

        self.store.snapshot(&TopicQueue::STATES, request)
    }
}
