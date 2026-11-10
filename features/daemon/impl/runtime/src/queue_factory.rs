use anyq_daemon_api::QueueServices;
use anyq_queue_api::{QueuePolicy, Result};
use std::{
    path::Path,
    sync::{Arc, atomic::AtomicBool},
};

pub trait QueueFactory: Send + Sync {
    fn observe(
        &self,
        path: &Path,
        cancelled: Arc<AtomicBool>,
    ) -> Result<Box<dyn anyq_queue_api::ObserveQueue>>;

    fn open(
        &self,
        path: &Path,
        policy: QueuePolicy,
        cancelled: Arc<AtomicBool>,
    ) -> Result<QueueServices>;
}
