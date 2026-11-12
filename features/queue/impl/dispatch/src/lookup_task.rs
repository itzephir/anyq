use crate::ports::UnitOfWork;
use crate::record_event::task_event;
use anyq_queue_api::QueueEventKind;
use anyq_queue_api::{QueuePolicy, Result, Task, parse_name};

pub(crate) fn lookup_task(
    records: &mut dyn UnitOfWork,
    input: &str,
    policy: QueuePolicy,
) -> Result<Task> {
    let name = parse_name(input)?;

    if let Some(task) = records.tasks().task(&name)? {
        return Ok(task);
    }

    policy.autocreate.require_task_creation(&name)?;
    let task = records
        .tasks()
        .insert_task(&name)?
        .with_start_policy(policy.task_autostart, false);

    if !policy.task_autostart {
        records.tasks().save_task(&task)?;
    }

    task_event(records, QueueEventKind::TaskRegistered, &task)?;

    Ok(task)
}
