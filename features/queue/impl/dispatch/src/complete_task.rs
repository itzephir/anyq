use crate::admit_queued_tasks::admit_queued_tasks;
use crate::record_event::task_event;
use crate::{lookup_task::lookup_task, ports::Store};
use anyq_queue_api::AutoCreatePolicy;
use anyq_queue_api::QueueEventKind;
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
        self.store
            .transaction(|records| complete_in_transaction(records, name, force, self.policy))
    }
}

pub(crate) fn complete_in_transaction(
    records: &mut dyn crate::ports::UnitOfWork,
    name: &str,
    force: bool,
    policy: QueuePolicy,
) -> Result<Task> {
    let mut task = lookup_task(records, name, policy)?;
    let work = records.work().managed(None, Some(task.name()))?;

    task.require_completion_permit(work.iter().map(|work| work.state))?;
    task.complete(force)?;
    records.tasks().save_task(&task)?;
    task_event(records, QueueEventKind::TaskCompleted, &task)?;
    admit_queued_tasks(records)?;

    Ok(task)
}
