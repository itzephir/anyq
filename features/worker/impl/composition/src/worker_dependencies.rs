use anyq_queue_api::{ClaimDelivery, CompleteDelivery, GetTopic, RejectDelivery, RenewLease};
/// Explicit composition-root dependencies, each exposing only one scenario.
pub struct WorkerDependencies {
    pub claim: Box<dyn ClaimDelivery>,
    pub renew: Box<dyn RenewLease>,
    pub complete: Box<dyn CompleteDelivery>,
    pub reject: Box<dyn RejectDelivery>,
    pub topics: Box<dyn GetTopic>,
}
