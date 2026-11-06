use crate::lookup_delivery::lookup_delivery;
use crate::ports::*;
use crate::publish_in_transaction::publish_in_transaction;
use anyq_queue_api::*;

pub struct CompleteDeliveryInteractor<S, C> {
    store: S,
    clock: C,
}

impl<S, C> CompleteDeliveryInteractor<S, C> {
    pub fn new(store: S, clock: C) -> Self {
        Self { store, clock }
    }
}

impl<S: Store, C: Clock> CompleteDelivery for CompleteDeliveryInteractor<S, C> {
    fn complete(&mut self, request: CompleteDeliveryRequest) -> Result<Completion> {
        let clock = &self.clock;

        self.store.transaction(|records| {
            let now = clock.now_ms();
            let mut delivery =
                lookup_delivery(records, request.lease.message_id, request.lease.group)?;

            delivery.ack(&request.lease.token, now, request.result)?;

            let next = request
                .next
                .map(|request| publish_in_transaction(records, &request, now))
                .transpose()?;

            records.deliveries().put_delivery(&delivery)?;

            Ok(Completion { delivery, next })
        })
    }
}
