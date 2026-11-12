use crate::record_event::task_event;
use crate::{lookup_task::lookup_task, ports::Store};
use anyq_queue_api::QueueEventKind;
use anyq_queue_api::{
    Metadata, QueuePolicy, ReplaceTaskMetadata, ReplaceTaskMetadataRequest, Result,
};

pub struct ReplaceTaskMetadataInteractor<S> {
    store: S,
    policy: QueuePolicy,
}

impl<S> ReplaceTaskMetadataInteractor<S> {
    pub fn new(store: S, policy: QueuePolicy) -> Self {
        Self { store, policy }
    }
}

impl<S: Store> ReplaceTaskMetadata for ReplaceTaskMetadataInteractor<S> {
    fn replace(&mut self, request: ReplaceTaskMetadataRequest) -> Result<Metadata> {
        self.store.transaction(|records| {
            let mut entity = lookup_task(records, &request.task, self.policy)?;

            let previous = entity.metadata().clone();

            entity.replace_metadata(request.metadata)?;

            records.tasks().save_task(&entity)?;

            if &previous != entity.metadata() {
                task_event(records, QueueEventKind::TaskMetadataChanged, &entity)?;
            }

            Ok(entity.metadata().clone())
        })
    }
}
