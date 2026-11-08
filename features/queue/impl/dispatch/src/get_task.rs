use crate::{lookup_task::lookup_task, ports::Store};
use anyq_queue_api::{GetTask, Result, Task};

pub struct GetTaskInteractor<S> {
    store: S,
}

impl<S> GetTaskInteractor<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S: Store> GetTask for GetTaskInteractor<S> {
    fn get(&mut self, name: &str) -> Result<Task> {
        self.store.transaction(|records| lookup_task(records, name))
    }
}
