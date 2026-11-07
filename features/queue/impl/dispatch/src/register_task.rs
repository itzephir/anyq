use crate::ports::Store;
use anyq_queue_api::{Error, RegisterTask, Result, Task, validate_name};

pub struct RegisterTaskInteractor<S> {
    store: S,
}

impl<S> RegisterTaskInteractor<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S: Store> RegisterTask for RegisterTaskInteractor<S> {
    fn register(&mut self, name: String) -> Result<Task> {
        validate_name(&name)?;

        self.store.transaction(|records| {
            if records.tasks().task(&name)?.is_some() {
                return Err(Error::Conflict(format!(
                    "task '{name}' is already registered"
                )));
            }

            records.tasks().insert_task(&name)
        })
    }
}
