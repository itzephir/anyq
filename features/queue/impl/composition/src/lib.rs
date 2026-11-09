mod queue_module;

use anyq_queue_api::{AutoCreatePolicy, Result};
use anyq_queue_dispatch::*;
use anyq_queue_sqlite::Sqlite;
pub use queue_module::QueueModule;
use std::{
    path::Path,
    sync::{Arc, atomic::AtomicBool},
};

pub fn open(path: &Path) -> Result<QueueModule> {
    open_with_options(
        path,
        AutoCreatePolicy::default(),
        Arc::new(AtomicBool::new(false)),
    )
}

pub fn open_with_options(
    path: &Path,
    policy: AutoCreatePolicy,
    cancelled: Arc<AtomicBool>,
) -> Result<QueueModule> {
    let store = Sqlite::open_with_cancellation(path, cancelled)?;

    Ok(QueueModule {
        create_topic: Box::new(CreateTopicInteractor::new(store.clone())),
        set_topic_capacity: Box::new(SetTopicCapacityInteractor::with_policy(
            store.clone(),
            policy,
        )),
        list_topics: Box::new(ListTopicsInteractor::new(store.clone())),
        list_tasks: Box::new(ListTasksInteractor::new(store.clone())),
        get_topic: Box::new(GetTopicInteractor::with_policy(store.clone(), policy)),
        get_task: Box::new(GetTaskInteractor::with_policy(store.clone(), policy)),
        get_topic_queue: Box::new(GetTopicQueueInteractor::with_policy(store.clone(), policy)),
        register_task: Box::new(RegisterTaskInteractor::with_policy(store.clone(), policy)),
        add_task_to_topic: Box::new(AddTaskToTopicInteractor::with_policy(store.clone(), policy)),
        try_start_task: Box::new(TryStartTaskInteractor::with_policy(store.clone(), policy)),
        complete_task: Box::new(CompleteTaskInteractor::with_policy(store, policy)),
    })
}
