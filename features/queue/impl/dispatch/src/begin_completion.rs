use crate::ports::Store;
use anyq_queue_api::*;

pub struct BeginCompletionInteractor<S> {
    store: S,
}

impl<S> BeginCompletionInteractor<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S: Store> BeginCompletion for BeginCompletionInteractor<S> {
    fn begin(&mut self, request: BeginCompletionRequest) -> Result<bool> {
        self.store.transaction(|records| {
            let task =
                crate::lookup_task::lookup_task(records, &request.task, QueuePolicy::default())?;
            let mut validated = task.clone();
            validated.complete(request.force)?;
            records.work().request_completion(task.name())?;
            let work = records.work().managed(&ManagedWorkFilter {
                task: Some(task.name().into()),
                states: ManagedWorkState::BLOCKING_STATES.to_vec(),
                ..Default::default()
            })?;

            Ok(work.iter().any(|work| work.state.blocks_completion()))
        })
    }
}
