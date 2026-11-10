use crate::ports::Store;
use anyq_queue_api::{CancelManagedWork, Error, ManagedWork, Result};

pub struct CancelManagedWorkInteractor<S> {
    store: S,
}

impl<S> CancelManagedWorkInteractor<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S: Store> CancelManagedWork for CancelManagedWorkInteractor<S> {
    fn cancel(&mut self, owner: &str, action_id: &str) -> Result<ManagedWork> {
        self.store.transaction(|records| {
            let mut work = records
                .work()
                .managed(Some(owner), None)?
                .into_iter()
                .find(|work| work.action_id == action_id)
                .ok_or_else(|| Error::NotFound("managed action is absent for this owner".into()))?;
            work.cancellation_requested = true;
            records.work().save_managed(&work)?;

            Ok(work)
        })
    }
}
