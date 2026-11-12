pub use anyq_queue_api::QueueServices as QueueModule;
use anyq_queue_api::{QueuePolicy, Result};
use anyq_queue_dispatch::*;
use anyq_queue_sqlite::Sqlite;
use std::{
    path::Path,
    sync::{Arc, atomic::AtomicBool},
};

pub fn open(path: &Path) -> Result<QueueModule> {
    open_with_options(
        path,
        QueuePolicy::default(),
        Arc::new(AtomicBool::new(false)),
    )
}

pub fn open_with_options(
    path: &Path,
    policy: QueuePolicy,
    cancelled: Arc<AtomicBool>,
) -> Result<QueueModule> {
    let store = Sqlite::open_with_cancellation(path, cancelled)?;

    Ok(QueueModule {
        get_task_metadata: Box::new(GetTaskMetadataInteractor::new(store.clone(), policy)),
        set_task_metadata: Box::new(SetTaskMetadataInteractor::new(store.clone(), policy)),
        unset_task_metadata: Box::new(UnsetTaskMetadataInteractor::new(store.clone(), policy)),
        replace_task_metadata: Box::new(ReplaceTaskMetadataInteractor::new(store.clone(), policy)),
        get_topic_metadata: Box::new(GetTopicMetadataInteractor::new(store.clone(), policy)),
        set_topic_metadata: Box::new(SetTopicMetadataInteractor::new(store.clone(), policy)),
        unset_topic_metadata: Box::new(UnsetTopicMetadataInteractor::new(store.clone(), policy)),
        replace_topic_metadata: Box::new(ReplaceTopicMetadataInteractor::new(
            store.clone(),
            policy,
        )),
        create_topic: Box::new(CreateTopicInteractor::new(store.clone())),
        set_topic_capacity: Box::new(SetTopicCapacityInteractor::with_policy(
            store.clone(),
            policy.autocreate,
        )),
        list_topics: Box::new(ListTopicsInteractor::new(store.clone())),
        list_tasks: Box::new(ListTasksInteractor::new(store.clone())),
        get_topic: Box::new(GetTopicInteractor::with_policy(
            store.clone(),
            policy.autocreate,
        )),
        get_task: Box::new(GetTaskInteractor::with_options(store.clone(), policy)),
        get_topic_queue: Box::new(GetTopicQueueInteractor::with_policy(
            store.clone(),
            policy.autocreate,
        )),
        register_task: Box::new(RegisterTaskInteractor::with_options(store.clone(), policy)),
        add_task_to_topics: Box::new(AddTaskToTopicsInteractor::with_options(
            store.clone(),
            policy,
        )),
        add_tasks_to_topic: Box::new(AddTasksToTopicInteractor::with_options(
            store.clone(),
            policy,
        )),
        add_task_to_topic: Box::new(AddTaskToTopicInteractor::with_options(
            store.clone(),
            policy,
        )),
        check_task_running: Box::new(CheckTaskRunningInteractor::with_options(
            store.clone(),
            policy,
        )),
        start_task: Box::new(StartTaskInteractor::with_options(store.clone(), policy)),
        set_topic_autostart: Box::new(SetTopicAutostartInteractor::with_policy(
            store.clone(),
            policy.autocreate,
        )),
        complete_task: Box::new(CompleteTaskInteractor::with_options(store, policy)),
    })
}
