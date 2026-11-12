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

            records
                .work()
                .managed(request.owner.as_deref(), task.as_deref())
        })
    }
}
