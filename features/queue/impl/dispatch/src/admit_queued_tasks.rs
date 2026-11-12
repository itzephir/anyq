use crate::ports::{TaskFilter, UnitOfWork};
use crate::record_event::task_event;
use anyq_queue_api::QueueEventKind;
use anyq_queue_api::{Result, StartOutcome, Task, TaskState};

pub(crate) fn admit_queued_tasks(records: &mut dyn UnitOfWork) -> Result<()> {
    let mut queued = records.tasks().tasks(&TaskFilter {
        topic: None,
        state: TaskState::Queued,
        before_sequence: None,
    })?;
    queued.sort_by_key(Task::sequence);

    for mut task in queued {
        let resources = crate::resource_availability::resource_availability(records, &task)?;
        let topics = resources
            .iter()
            .map(|resource| resource.topic.clone())
            .collect::<Vec<_>>();

        if !task.permits_autostart(&topics) {
            continue;
        }

        if task.try_start(&resources)? == StartOutcome::Started {
            records.tasks().save_task(&task)?;
            task_event(records, QueueEventKind::TaskRunning, &task)?;
        }
    }

    Ok(())
}
