use anyq_queue_api::{ClaimDelivery, CompleteDelivery, GetTopic, RejectDelivery, RenewLease};

/// Explicit consumer-owned dependencies, each exposing only one scenario.
pub struct ExecutionPorts {
    pub claim: Box<dyn ClaimDelivery>,
    pub renew: Box<dyn RenewLease>,
    pub complete: Box<dyn CompleteDelivery>,
    pub reject: Box<dyn RejectDelivery>,
    pub topics: Box<dyn GetTopic>,
}
