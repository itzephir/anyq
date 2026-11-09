use crate::{lookup_task::lookup_task, ports::Store};
use anyq_queue_api::AutoCreatePolicy;
use anyq_queue_api::{CompleteTask, Result, Task};

pub struct CompleteTaskInteractor<S> {
    store: S,
    policy: AutoCreatePolicy,
}

impl<S> CompleteTaskInteractor<S> {
    pub fn new(store: S) -> Self {
        Self::with_policy(store, AutoCreatePolicy::default())
    }

    pub fn with_policy(store: S, policy: AutoCreatePolicy) -> Self {
        Self { store, policy }
    }
}

impl<S: Store> CompleteTask for CompleteTaskInteractor<S> {
    fn complete(&mut self, name: &str, force: bool) -> Result<Task> {
        self.store.transaction(|records| {
            let mut task = lookup_task(records, name, self.policy)?;

            task.complete(force)?;
            records.tasks().save_task(&task)?;

            Ok(task)
        })
    }
}
