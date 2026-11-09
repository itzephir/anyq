use crate::admit_queued_tasks::admit_queued_tasks;
use crate::{lookup_task::lookup_task, ports::Store};
use anyq_queue_api::AutoCreatePolicy;
use anyq_queue_api::QueuePolicy;
use anyq_queue_api::{CompleteTask, Result, Task};

pub struct CompleteTaskInteractor<S> {
    store: S,
    policy: QueuePolicy,
}

impl<S> CompleteTaskInteractor<S> {
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

impl<S: Store> CompleteTask for CompleteTaskInteractor<S> {
    fn complete(&mut self, name: &str, force: bool) -> Result<Task> {
        self.store.transaction(|records| {
            let mut task = lookup_task(records, name, self.policy)?;

            task.complete(force)?;
            records.tasks().save_task(&task)?;

            admit_queued_tasks(records)?;

            Ok(task)
        })
    }
}
