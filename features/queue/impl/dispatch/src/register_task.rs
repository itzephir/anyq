use crate::{lookup_topic::lookup_topic, ports::Store};
use anyq_queue_api::{Error, RegisterTask, RegisterTaskRequest, Result, Task, validate_name};

pub struct RegisterTaskInteractor<S> {
    store: S,
}

impl<S> RegisterTaskInteractor<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S: Store> RegisterTask for RegisterTaskInteractor<S> {
    fn register(&mut self, request: RegisterTaskRequest) -> Result<Task> {
        validate_name(&request.name)?;

        self.store.transaction(|records| {
            if records.tasks().task(&request.name)?.is_some() {
                return Err(Error::Conflict(format!(
                    "task '{}' is already registered",
                    request.name
                )));
            }

            for topic in &request.topics {
                lookup_topic(records, topic)?;
            }

            let mut task = records.tasks().insert_task(&request.name)?;

            for topic in &request.topics {
                task.add_topic(topic)?;
            }

            if !request.topics.is_empty() {
                records.tasks().save_task(&task)?;
            }

            Ok(task)
        })
    }
}
