//! Explicit local client/server composition roots.
mod sqlite_queue_factory;

use anyq_daemon_api::QueueServices;
use anyq_daemon_client::*;
use anyq_daemon_transport::Endpoint;
use anyq_queue_api::{Error, QueuePolicy, Result};
use std::{
    path::Path,
    sync::{Arc, atomic::AtomicBool},
};

fn endpoint(path: &Path) -> Result<Endpoint> {
    Endpoint::resolve(path).map_err(|error| Error::Storage(error.to_string()))
}

pub fn observe(
    path: &Path,
    cancelled: Arc<AtomicBool>,
) -> Result<Box<dyn anyq_queue_api::ObserveQueue>> {
    let endpoint = endpoint(path)?;
    let channel = Channel::open_observer(&endpoint, cancelled)?;

    Ok(Box::new(RemoteObserveQueue::new(
        channel,
        endpoint.identity,
    )))
}

pub fn connect(
    path: &Path,
    policy: QueuePolicy,
    cancelled: Arc<AtomicBool>,
) -> Result<QueueServices> {
    let channel = Channel::open(&endpoint(path)?, policy, cancelled)?;

    Ok(QueueServices {
        create_topic: Box::new(RemoteCreateTopic::new(channel.clone())),
        set_topic_capacity: Box::new(RemoteSetTopicCapacity::new(channel.clone())),
        list_topics: Box::new(RemoteListTopics::new(channel.clone())),
        list_tasks: Box::new(RemoteListTasks::new(channel.clone())),
        get_topic: Box::new(RemoteGetTopic::new(channel.clone())),
        get_task: Box::new(RemoteGetTask::new(channel.clone())),
        get_topic_queue: Box::new(RemoteGetTopicQueue::new(channel.clone())),
        register_task: Box::new(RemoteRegisterTask::new(channel.clone())),
        add_task_to_topics: Box::new(RemoteAddTaskToTopics::new(channel.clone())),
        add_tasks_to_topic: Box::new(RemoteAddTasksToTopic::new(channel.clone())),
        add_task_to_topic: Box::new(RemoteAddTaskToTopic::new(channel.clone())),
        check_task_running: Box::new(RemoteCheckTaskRunning::new(channel.clone())),
        start_task: Box::new(RemoteStartTask::new(channel.clone())),
        set_topic_autostart: Box::new(RemoteSetTopicAutostart::new(channel.clone())),
        complete_task: Box::new(RemoteCompleteTask::new(channel.clone())),
    })
}

pub fn run(path: &Path, cancelled: Arc<AtomicBool>) -> Result<()> {
    anyq_daemon_runtime::serve(
        endpoint(path)?,
        Arc::new(sqlite_queue_factory::SqliteQueueFactory),
        cancelled,
    )
}

pub fn daemon_status(path: &Path, cancelled: Arc<AtomicBool>) -> Result<Option<u32>> {
    status(&endpoint(path)?, cancelled)
}

pub fn daemon_stop(path: &Path, cancelled: Arc<AtomicBool>) -> Result<()> {
    stop(&endpoint(path)?, cancelled)
}

pub fn daemon_start(path: &Path, policy: QueuePolicy, cancelled: Arc<AtomicBool>) -> Result<u32> {
    let _services = connect(path, policy, cancelled.clone())?;

    daemon_status(path, cancelled)?
        .ok_or_else(|| Error::Storage("daemon exited during startup".into()))
}
