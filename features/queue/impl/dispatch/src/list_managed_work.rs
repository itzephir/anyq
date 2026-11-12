use crate::ports::Store;
use anyq_queue_api::*;

pub struct ListManagedWorkInteractor<S> {
    store: S,
}

impl<S> ListManagedWorkInteractor<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S: Store> ListManagedWork for ListManagedWorkInteractor<S> {
    fn list(&mut self, request: ListManagedWorkRequest) -> Result<Vec<ManagedWork>> {
        self.store.transaction(|records| {
            let task = request.task.as_deref().map(parse_name).transpose()?;

            let filter = ManagedWorkFilter {
                owner: request.owner,
                task,
                action: None,
                after: request.after,
                limit: request.limit,
                states: request.states,
            };
            filter.validate()?;

            records.work().managed(&filter)
        })
    }
}
