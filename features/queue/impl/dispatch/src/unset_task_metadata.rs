use crate::record_event::task_event;
use crate::{lookup_task::lookup_task, ports::Store};
use anyq_queue_api::QueueEventKind;
use anyq_queue_api::{Metadata, QueuePolicy, Result, UnsetTaskMetadata, UnsetTaskMetadataRequest};

pub struct UnsetTaskMetadataInteractor<S> {
    store: S,
    policy: QueuePolicy,
}

impl<S> UnsetTaskMetadataInteractor<S> {
    pub fn new(store: S, policy: QueuePolicy) -> Self {
        Self { store, policy }
    }
}

impl<S: Store> UnsetTaskMetadata for UnsetTaskMetadataInteractor<S> {
    fn unset(&mut self, request: UnsetTaskMetadataRequest) -> Result<Metadata> {
        self.store.transaction(|records| {
            let mut entity = lookup_task(records, &request.task, self.policy)?;

            let previous = entity.metadata().clone();

            entity.unset_metadata_key(&request.key)?;

            records.tasks().save_task(&entity)?;

            if &previous != entity.metadata() {
                task_event(records, QueueEventKind::TaskMetadataChanged, &entity)?;
            }

            Ok(entity.metadata().clone())
        })
    }
}
