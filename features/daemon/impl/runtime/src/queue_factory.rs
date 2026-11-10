use anyq_daemon_api::DaemonServices;
use anyq_queue_api::{QueuePolicy, Result};
use std::{
    path::Path,
    sync::{Arc, atomic::AtomicBool},
};

pub trait QueueFactory: Send + Sync {
    fn background(&self, path: &Path, shutdown: Arc<AtomicBool>) -> Result<()>;

    fn open(
        &self,
        path: &Path,
        policy: QueuePolicy,
        cancelled: Arc<AtomicBool>,
        directory: anyq_queue_api::ExecutionDirectory,
    ) -> Result<DaemonServices>;
}
