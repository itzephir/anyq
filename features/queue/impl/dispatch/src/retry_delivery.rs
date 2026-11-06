use crate::lookup_delivery::lookup_delivery;
use crate::ports::*;
use crate::recover_expired::recover_expired;
use anyq_queue_api::*;

pub struct RetryDeliveryInteractor<S, C> {
    store: S,
    clock: C,
}

impl<S, C> RetryDeliveryInteractor<S, C> {
    pub fn new(store: S, clock: C) -> Self {
        Self { store, clock }
    }
}

impl<S: Store, C: Clock> RetryDelivery for RetryDeliveryInteractor<S, C> {
    fn retry(&mut self, message_id: i64, group: String) -> Result<Delivery> {
        let clock = &self.clock;

        self.store.transaction(|records| {
            let now = clock.now_ms();
            let message = records
                .messages()
                .message(message_id)?
                .ok_or_else(|| Error::NotFound("message does not exist".into()))?;

            recover_expired(records, Some(message.topic), now)?;

            let mut delivery = lookup_delivery(records, message_id, group)?;
            delivery.retry(now)?;
            records.deliveries().put_delivery(&delivery)?;

            Ok(delivery)
        })
    }
}
