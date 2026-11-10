use crate::{lookup_task::lookup_task, ports::Store};
use anyq_queue_api::{Metadata, QueuePolicy, Result, SetTaskMetadata, SetTaskMetadataRequest};

pub struct SetTaskMetadataInteractor<S> {
    store: S,
    policy: QueuePolicy,
}

impl<S> SetTaskMetadataInteractor<S> {
    pub fn new(store: S, policy: QueuePolicy) -> Self {
        Self { store, policy }
    }
}

impl<S: Store> SetTaskMetadata for SetTaskMetadataInteractor<S> {
    fn set(&mut self, request: SetTaskMetadataRequest) -> Result<Metadata> {
        self.store.transaction(|records| {
            let mut entity = lookup_task(records, &request.task, self.policy)?;

            entity.set_metadata_key(&request.key, request.value)?;

            records.tasks().save_task(&entity)?;

            Ok(entity.metadata().clone())
        })
    }
}
