use crate::lookup_delivery::lookup_delivery;
use crate::ports::*;
use anyq_queue_api::*;

pub struct RejectDeliveryInteractor<S, C> {
    store: S,
    clock: C,
}

impl<S, C> RejectDeliveryInteractor<S, C> {
    pub fn new(store: S, clock: C) -> Self {
        Self { store, clock }
    }
}

impl<S: Store, C: Clock> RejectDelivery for RejectDeliveryInteractor<S, C> {
    fn reject(&mut self, request: RejectDeliveryRequest) -> Result<Delivery> {
        let clock = &self.clock;

        self.store.transaction(|records| {
            let now = clock.now_ms();
            let mut delivery =
                lookup_delivery(records, request.lease.message_id, request.lease.group)?;

            delivery.nack(&request.lease.token, now, request.delay_ms, request.reason)?;
            records.deliveries().put_delivery(&delivery)?;

            Ok(delivery)
        })
    }
}
