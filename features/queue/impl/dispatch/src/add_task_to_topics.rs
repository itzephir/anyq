use crate::{link_tasks_to_topics::link_tasks_to_topics, ports::Store};
use anyq_queue_api::QueuePolicy;
use anyq_queue_api::{
    AddTaskToTopics, AddTaskToTopicsRequest, AutoCreatePolicy, Error, Result, Task,
};

pub struct AddTaskToTopicsInteractor<S> {
    store: S,
    policy: QueuePolicy,
}

impl<S> AddTaskToTopicsInteractor<S> {
    pub fn new(store: S) -> Self {
        Self::with_policy(store, AutoCreatePolicy::default())
    }

    pub fn with_policy(store: S, policy: AutoCreatePolicy) -> Self {
        Self::with_options(
            store,
            QueuePolicy {
                autocreate: policy,
                ..QueuePolicy::default()
            },
        )
    }

    pub fn with_options(store: S, policy: QueuePolicy) -> Self {
        Self { store, policy }
    }
}

impl<S: Store> AddTaskToTopics for AddTaskToTopicsInteractor<S> {
    fn add(&mut self, request: AddTaskToTopicsRequest) -> Result<Task> {
        self.store.transaction(|records| {
            link_tasks_to_topics(records, &[request.task], &request.topics, self.policy)?
                .pop()
                .ok_or_else(|| Error::Storage("linked task disappeared".into()))
        })
    }
}
