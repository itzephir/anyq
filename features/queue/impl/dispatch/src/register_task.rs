use crate::record_event::task_event;
use crate::{admit_queued_tasks::admit_queued_tasks, resource_availability::resource_availability};
use crate::{lookup_topic::lookup_topic, ports::Store};
use anyq_queue_api::AutoCreatePolicy;
use anyq_queue_api::QueueEventKind;
use anyq_queue_api::QueuePolicy;
use anyq_queue_api::{
    Error, RegisterTask, RegisterTaskRequest, Result, StartOutcome, Task, parse_name,
};

pub struct RegisterTaskInteractor<S> {
    store: S,
    policy: QueuePolicy,
}

impl<S> RegisterTaskInteractor<S> {
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

impl<S: Store> RegisterTask for RegisterTaskInteractor<S> {
    fn register(&mut self, request: RegisterTaskRequest) -> Result<Task> {
        let name = parse_name(&request.name)?;

        self.store.transaction(|records| {
            if records.tasks().task(&name)?.is_some() {
                return Err(Error::Conflict(format!(
                    "task '{}' is already registered",
                    name
                )));
            }

            let topics = request
                .topics
                .iter()
                .map(|topic| lookup_topic(records, topic, self.policy.autocreate))
                .collect::<Result<Vec<_>>>()?;

            let mut task = records
                .tasks()
                .insert_task(&name)?
                .with_start_policy(self.policy.task_autostart, false);

            task.replace_metadata(request.metadata)?;

            task_event(records, QueueEventKind::TaskRegistered, &task)?;

            for topic in &topics {
                let is_new = !task.topics().contains(&topic.name);
                task.link_topic(topic)?;

                if is_new {
                    task_event(records, QueueEventKind::TaskLinked, &task)?;
                }
            }

            let explicit = task.configure_start(request.start, &topics)?;
            records.tasks().save_task(&task)?;

            if explicit {
                let resources = resource_availability(records, &task)?;

                if task.try_start(&resources)? == StartOutcome::Busy {
                    return Err(Error::Conflict(format!(
                        "'{}' cannot start: resources or queue position are unavailable",
                        task.name()
                    )));
                }

                records.tasks().save_task(&task)?;
                task_event(records, QueueEventKind::TaskRunning, &task)?;
            }

            admit_queued_tasks(records)?;

            records.tasks().task(task.name())?.ok_or_else(|| {
                Error::Storage("registered task disappeared inside a transaction".into())
            })
        })
    }
}
