use crate::{
    lookup_task::lookup_task,
    lookup_topic::lookup_topic_value,
    ports::{Store, TaskFilter},
};
use anyq_queue_api::AutoCreatePolicy;
use anyq_queue_api::{ResourceAvailability, Result, StartOutcome, TaskState, TryStartTask};

pub struct TryStartTaskInteractor<S> {
    store: S,
    policy: AutoCreatePolicy,
}

impl<S> TryStartTaskInteractor<S> {
    pub fn new(store: S) -> Self {
        Self::with_policy(store, AutoCreatePolicy::default())
    }

    pub fn with_policy(store: S, policy: AutoCreatePolicy) -> Self {
        Self { store, policy }
    }
}

impl<S: Store> TryStartTask for TryStartTaskInteractor<S> {
    fn try_start(&mut self, name: &str) -> Result<StartOutcome> {
        self.store.transaction(|records| {
            let mut task = lookup_task(records, name, self.policy)?;
            task.require_queued()?;

            let mut resources = Vec::new();

            for name in task.topics() {
                let topic = lookup_topic_value(records, name, self.policy)?;
                let running = records.tasks().count_tasks(&TaskFilter {
                    topic: name.clone(),
                    state: TaskState::Running,
                    before_sequence: None,
                })?;
                let queued_before = records.tasks().count_tasks(&TaskFilter {
                    topic: name.clone(),
                    state: TaskState::Queued,
                    before_sequence: Some(task.sequence()),
                })?;

                resources.push(ResourceAvailability {
                    topic,
                    running,
                    queued_before,
                });
            }

            let outcome = task.try_start(&resources)?;

            if outcome == StartOutcome::Started {
                records.tasks().save_task(&task)?;
            }

            Ok(outcome)
        })
    }
}
