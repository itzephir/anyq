use crate::*;

pub trait RenewLease {
    fn renew(&mut self, lease: Lease, lease_ms: i64) -> Result<Delivery>;
}
