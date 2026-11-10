use crate::{lookup_task::lookup_task, ports::Store};
use anyq_queue_api::QueuePolicy;
use anyq_queue_api::{AutoCreatePolicy, CheckTaskRunning, Result};

pub struct CheckTaskRunningInteractor<S> {
    store: S,
    policy: QueuePolicy,
}

impl<S> CheckTaskRunningInteractor<S> {
    pub fn new(store: S) -> Self {
        Self::with_policy(store, AutoCreatePolicy::default())
    }

    pub fn with_policy(store: S, policy: AutoCreatePolicy) -> Self {
        Self::with_options(
            store,
            QueuePolicy {
                autocreate: policy,
                ..QueuePolicy::default()
            },
        )
    }

    pub fn with_options(store: S, policy: QueuePolicy) -> Self {
        Self { store, policy }
    }
}

impl<S: Store> CheckTaskRunning for CheckTaskRunningInteractor<S> {
    fn check_running(&mut self, name: &str) -> Result<bool> {
        self.store
            .transaction(|records| lookup_task(records, name, self.policy)?.check_running())
    }
}
