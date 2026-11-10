use crate::{link_tasks_to_topics::link_tasks_to_topics, ports::Store};
use anyq_queue_api::QueuePolicy;
use anyq_queue_api::{AddTasksToTopic, AddTasksToTopicRequest, AutoCreatePolicy, Result, Task};

pub struct AddTasksToTopicInteractor<S> {
    store: S,
    policy: QueuePolicy,
}

impl<S> AddTasksToTopicInteractor<S> {
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

impl<S: Store> AddTasksToTopic for AddTasksToTopicInteractor<S> {
    fn add(&mut self, request: AddTasksToTopicRequest) -> Result<Vec<Task>> {
        self.store.transaction(|records| {
            link_tasks_to_topics(records, &request.tasks, &[request.topic], self.policy)
        })
    }
}
