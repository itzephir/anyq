mod queue_module;

use anyq_queue_api::Result;
use anyq_queue_dispatch::*;
use anyq_queue_sqlite::Sqlite;
pub use queue_module::QueueModule;
use std::path::Path;

pub fn open(path: &Path) -> Result<QueueModule> {
    let store = Sqlite::open(path)?;

    Ok(QueueModule {
        create_topic: Box::new(CreateTopicInteractor::new(store.clone())),
        set_topic_capacity: Box::new(SetTopicCapacityInteractor::new(store.clone())),
        list_topics: Box::new(ListTopicsInteractor::new(store.clone())),
        list_tasks: Box::new(ListTasksInteractor::new(store.clone())),
        get_topic: Box::new(GetTopicInteractor::new(store.clone())),
        get_task: Box::new(GetTaskInteractor::new(store.clone())),
        get_topic_queue: Box::new(GetTopicQueueInteractor::new(store.clone())),
        register_task: Box::new(RegisterTaskInteractor::new(store.clone())),
        add_task_to_topic: Box::new(AddTaskToTopicInteractor::new(store.clone())),
        try_start_task: Box::new(TryStartTaskInteractor::new(store.clone())),
        complete_task: Box::new(CompleteTaskInteractor::new(store)),
    })
}
