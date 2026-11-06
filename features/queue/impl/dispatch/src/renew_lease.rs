use crate::lookup_delivery::lookup_delivery;
use crate::ports::*;
use anyq_queue_api::*;

pub struct RenewLeaseInteractor<S, C> {
    store: S,
    clock: C,
}
impl<S, C> RenewLeaseInteractor<S, C> {
    pub fn new(store: S, clock: C) -> Self {
        Self { store, clock }
    }
}
impl<S: Store, C: Clock> RenewLease for RenewLeaseInteractor<S, C> {
    fn renew(&mut self, lease: Lease, lease_ms: i64) -> Result<Delivery> {
        let clock = &self.clock;
        self.store.transaction(|records| {
            let now = clock.now_ms();
            let mut delivery = lookup_delivery(records, lease.message_id, lease.group)?;
            delivery.renew(&lease.token, now, lease_ms)?;
            records.deliveries().put_delivery(&delivery)?;
            Ok(delivery)
        })
    }
}
