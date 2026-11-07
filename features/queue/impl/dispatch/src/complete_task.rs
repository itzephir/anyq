use crate::{lookup_task::lookup_task, ports::Store};
use anyq_queue_api::{CompleteTask, Result, Task};

pub struct CompleteTaskInteractor<S> {
    store: S,
}

impl<S> CompleteTaskInteractor<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S: Store> CompleteTask for CompleteTaskInteractor<S> {
    fn complete(&mut self, name: &str, force: bool) -> Result<Task> {
        self.store.transaction(|records| {
            let mut task = lookup_task(records, name)?;

            task.complete(force)?;
            records.tasks().save_task(&task)?;

            Ok(task)
        })
    }
}
