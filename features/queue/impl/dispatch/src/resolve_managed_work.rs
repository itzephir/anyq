use crate::ports::Store;
use anyq_queue_api::*;

pub struct ResolveManagedWorkInteractor<S> {
    store: S,
}

impl<S> ResolveManagedWorkInteractor<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S: Store> ResolveManagedWork for ResolveManagedWorkInteractor<S> {
    fn resolve(&mut self, request: ResolveManagedWorkRequest) -> Result<ManagedWork> {
        self.store.transaction(|records| {
            if !request.confirmed_cleanup {
                return Err(Error::Invalid(
                    "confirmed external cleanup is required".into(),
                ));
            }
            validate_action_identity(&request.owner, &request.action_id)?;
            let mut work = records
                .work()
                .managed(Some(&request.owner), None)?
                .into_iter()
                .find(|work| work.action_id == request.action_id)
                .ok_or_else(|| Error::NotFound("managed action is absent for this owner".into()))?;
            work.report(ManagedWorkState::Stopped)?;
            records.work().save_managed(&work)?;

            Ok(work)
        })
    }
}
