use crate::{
    admit_queued_tasks::admit_queued_tasks, lookup_task::lookup_task, ports::Store,
    resource_availability::resource_availability,
};
use anyq_queue_api::{QueuePolicy, Result, StartOutcome, StartTask, StartTaskOutcome};

pub struct StartTaskInteractor<S> {
    store: S,
    policy: QueuePolicy,
}

impl<S> StartTaskInteractor<S> {
    pub fn new(store: S) -> Self {
        Self::with_options(store, QueuePolicy::default())
    }

    pub fn with_options(store: S, policy: QueuePolicy) -> Self {
        Self { store, policy }
    }
}

impl<S: Store> StartTask for StartTaskInteractor<S> {
    fn start(&mut self, name: &str) -> Result<StartTaskOutcome> {
        self.store.transaction(|records| {
            let mut task = lookup_task(records, name, self.policy)?;

            if task.check_running()? {
                return Ok(StartTaskOutcome::AlreadyRunning);
            }

            let resources = resource_availability(records, &task)?;

            if task.try_start(&resources)? == StartOutcome::Busy {
                return Ok(StartTaskOutcome::Busy);
            }

            records.tasks().save_task(&task)?;
            admit_queued_tasks(records)?;

            Ok(StartTaskOutcome::Started)
        })
    }
}
