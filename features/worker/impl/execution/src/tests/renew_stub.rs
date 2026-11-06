use super::{assignment, trace::Trace};
use anyq_queue_api::{Delivery, Error, Lease, RenewLease, Result};
pub struct RenewStub {
    pub trace: Trace,
    pub fail: bool,
}
impl RenewLease for RenewStub {
    fn renew(&mut self, _: Lease, _: i64) -> Result<Delivery> {
        self.trace.renewals.set(self.trace.renewals.get() + 1);
        if self.fail {
            Err(Error::LeaseLost)
        } else {
            Ok(assignment().delivery)
        }
    }
}
