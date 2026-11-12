use crate::ports::UnitOfWork;
use anyq_queue_api::{Error, EventSnapshot, QueueEventKind, Result, Task, Topic};

pub(crate) fn task_event(
    records: &mut dyn UnitOfWork,
    kind: QueueEventKind,
    task: &Task,
) -> Result<()> {
    if kind == QueueEventKind::TaskRegistered {
        if let Some(directory) = records.caller_directory().cloned() {
            records.events().save_directory(task.name(), &directory)?;
        }
    }

    let topics = task
        .topics()
        .iter()
        .map(|name| {
            records
                .topics()
                .topic(name)?
                .ok_or_else(|| Error::Storage("event topic disappeared".into()))
        })
        .collect::<Result<_>>()?;
    let directory = records.events().directory(task.name())?;
    let snapshot = EventSnapshot {
        kind,
        occurred_at: records.clock().now_millis(),
        task: Some(task.clone()),
        topics,
        directory,
    };

    records.events().append(&snapshot)
}

pub(crate) fn topic_event(
    records: &mut dyn UnitOfWork,
    kind: QueueEventKind,
    topic: &Topic,
) -> Result<()> {
    let snapshot = EventSnapshot {
        kind,
        occurred_at: records.clock().now_millis(),
        task: None,
        topics: vec![topic.clone()],
        directory: None,
    };

    records.events().append(&snapshot)
}
