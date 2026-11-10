use crate::ports::Store;
use anyq_queue_api::*;

pub struct ReportManagedWorkInteractor<S> {
    store: S,
}

impl<S> ReportManagedWorkInteractor<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S: Store> ReportManagedWork for ReportManagedWorkInteractor<S> {
    fn report(&mut self, request: ReportManagedWorkRequest) -> Result<ManagedWork> {
        self.store.transaction(|records| {
            validate_action_identity(&request.owner, &request.action_id)?;
            let mut work = records
                .work()
                .managed(Some(&request.owner), None)?
                .into_iter()
                .find(|work| work.action_id == request.action_id)
                .ok_or_else(|| Error::NotFound("managed action is absent for this owner".into()))?;
            work.report(request.state)?;
            records.work().save_managed(&work)?;

            Ok(work)
        })
    }
}
