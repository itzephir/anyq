use anyq_daemon_api::QueueServices;
use anyq_queue_api::{QueuePolicy, Result};
use std::{
    path::Path,
    sync::{Arc, atomic::AtomicBool},
};

pub trait QueueFactory: Send + Sync {
    fn open(
        &self,
        path: &Path,
        policy: QueuePolicy,
        cancelled: Arc<AtomicBool>,
    ) -> Result<QueueServices>;
}
