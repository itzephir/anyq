use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RejectDeliveryRequest {
    pub lease: Lease,
    pub delay_ms: i64,
    pub reason: String,
}
