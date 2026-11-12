use crate::{lookup_task::lookup_task, ports::Store};
use anyq_queue_api::{GetTaskMetadata, GetTaskMetadataRequest, MetadataValue, QueuePolicy, Result};

pub struct GetTaskMetadataInteractor<S> {
    store: S,
    policy: QueuePolicy,
}

impl<S> GetTaskMetadataInteractor<S> {
    pub fn new(store: S, policy: QueuePolicy) -> Self {
        Self { store, policy }
    }
}

impl<S: Store> GetTaskMetadata for GetTaskMetadataInteractor<S> {
    fn get(&mut self, request: GetTaskMetadataRequest) -> Result<MetadataValue> {
        self.store.transaction(|records| {
            let entity = lookup_task(records, &request.task, self.policy)?;

            entity.metadata().get(request.key.as_deref()).cloned()
        })
    }
}
