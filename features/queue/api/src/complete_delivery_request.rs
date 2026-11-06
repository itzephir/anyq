use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompleteDeliveryRequest {
    pub lease: Lease,
    pub result: Option<String>,
    pub next: Option<Publish>,
}
