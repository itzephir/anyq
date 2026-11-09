use crate::{lookup_task::lookup_task, ports::Store};
use anyq_queue_api::AutoCreatePolicy;
use anyq_queue_api::{GetTask, Result, Task};

pub struct GetTaskInteractor<S> {
    store: S,
    policy: AutoCreatePolicy,
}

impl<S> GetTaskInteractor<S> {
    pub fn new(store: S) -> Self {
        Self::with_policy(store, AutoCreatePolicy::default())
    }

    pub fn with_policy(store: S, policy: AutoCreatePolicy) -> Self {
        Self { store, policy }
    }
}

impl<S: Store> GetTask for GetTaskInteractor<S> {
    fn get(&mut self, name: &str) -> Result<Task> {
        self.store
            .transaction(|records| lookup_task(records, name, self.policy))
    }
}
