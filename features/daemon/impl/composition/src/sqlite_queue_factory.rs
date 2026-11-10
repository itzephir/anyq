use anyq_daemon_api::QueueServices;
use anyq_daemon_runtime::QueueFactory;
use anyq_queue_api::{QueuePolicy, Result};
use std::{
    path::Path,
    sync::{Arc, atomic::AtomicBool},
};

pub(crate) struct SqliteQueueFactory;

impl QueueFactory for SqliteQueueFactory {
    fn observe(
        &self,
        path: &Path,
        cancelled: Arc<AtomicBool>,
    ) -> Result<Box<dyn anyq_queue_api::ObserveQueue>> {
        anyq_queue_impl::observe_existing(path, cancelled)
    }

    fn open(
        &self,
        path: &Path,
        policy: QueuePolicy,
        cancelled: Arc<AtomicBool>,
    ) -> Result<QueueServices> {
        anyq_queue_impl::open_with_options(path, policy, cancelled)
    }
}
