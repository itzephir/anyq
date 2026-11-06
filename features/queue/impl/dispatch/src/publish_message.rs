use crate::ports::*;
use crate::publish_in_transaction::publish_in_transaction;
use anyq_queue_api::*;

pub struct PublishMessageInteractor<S, C> {
    store: S,
    clock: C,
}
impl<S, C> PublishMessageInteractor<S, C> {
    pub fn new(store: S, clock: C) -> Self {
        Self { store, clock }
    }
}
impl<S: Store, C: Clock> PublishMessage for PublishMessageInteractor<S, C> {
    fn publish(&mut self, request: Publish) -> Result<Message> {
        let clock = &self.clock;
        self.store.transaction(|records| {
            let now = clock.now_ms();
            publish_in_transaction(records, &request, now)
        })
    }
}
