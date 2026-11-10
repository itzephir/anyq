use crate::record_event::task_event;
use crate::{
    admit_queued_tasks::admit_queued_tasks, lookup_task::lookup_task, lookup_topic::lookup_topic,
    ports::UnitOfWork,
};
use anyq_queue_api::QueueEventKind;
use anyq_queue_api::{Error, QueuePolicy, Result, Task};

pub(crate) fn link_tasks_to_topics(
    records: &mut dyn UnitOfWork,
    names: &[String],
    topics: &[String],
    policy: QueuePolicy,
) -> Result<Vec<Task>> {
    if names.is_empty() || topics.is_empty() {
        return Err(Error::Invalid("link requires tasks and topics".into()));
    }

    let mut tasks = names
        .iter()
        .map(|name| lookup_task(records, name, policy))
        .collect::<Result<Vec<_>>>()?;
    tasks.sort_by_key(Task::sequence);
    tasks.dedup_by_key(|task| task.sequence());
    let topics = topics
        .iter()
        .map(|topic| lookup_topic(records, topic, policy.autocreate))
        .collect::<Result<Vec<_>>>()?;

    for task in &mut tasks {
        for topic in &topics {
            let is_new = !task.topics().contains(&topic.name);
            task.link_topic(topic)?;

            if is_new {
                task_event(records, QueueEventKind::TaskLinked, task)?;
            }
        }

        records.tasks().save_task(task)?;
    }

    admit_queued_tasks(records)?;

    tasks
        .iter()
        .map(|task| {
            records.tasks().task(task.name())?.ok_or_else(|| {
                Error::Storage("linked task disappeared inside a transaction".into())
            })
        })
        .collect()
}
