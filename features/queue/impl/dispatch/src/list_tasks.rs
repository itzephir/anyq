use crate::ports::Store;
use anyq_queue_api::{ListTasks, Result};

pub struct ListTasksInteractor<S> {
    store: S,
}

impl<S> ListTasksInteractor<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S: Store> ListTasks for ListTasksInteractor<S> {
    fn list(&mut self) -> Result<Vec<String>> {
        self.store
            .transaction(|records| records.tasks().task_names())
    }
}
