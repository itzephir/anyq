use crate::{lookup_topic::lookup_topic, ports::Store};
use anyq_queue_api::AutoCreatePolicy;
use anyq_queue_api::{Error, RegisterTask, RegisterTaskRequest, Result, Task, parse_name};

pub struct RegisterTaskInteractor<S> {
    store: S,
    policy: AutoCreatePolicy,
}

impl<S> RegisterTaskInteractor<S> {
    pub fn new(store: S) -> Self {
        Self::with_policy(store, AutoCreatePolicy::default())
    }

    pub fn with_policy(store: S, policy: AutoCreatePolicy) -> Self {
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
                .map(|topic| lookup_topic(records, topic, self.policy))
                .collect::<Result<Vec<_>>>()?;

            let mut task = records.tasks().insert_task(&name)?;

            for topic in &topics {
                task.link_topic(topic)?;
            }

            if !request.topics.is_empty() {
                records.tasks().save_task(&task)?;
            }

            Ok(task)
        })
    }
}
