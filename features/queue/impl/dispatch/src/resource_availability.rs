use crate::ports::{TaskFilter, UnitOfWork};
use anyq_queue_api::{Error, ResourceAvailability, Result, Task, TaskState};

pub(crate) fn resource_availability(
    records: &mut dyn UnitOfWork,
    task: &Task,
) -> Result<Vec<ResourceAvailability>> {
    let mut resources = Vec::new();

    for name in task.topics() {
        let topic = records
            .topics()
            .topic(name)?
            .ok_or_else(|| Error::Storage(format!("missing task topic '{name}'")))?;
        let running = records.tasks().count_tasks(&TaskFilter {
            topic: Some(name.clone()),
            state: TaskState::Running,
            before_sequence: None,
        })?;
        let queued_before = records.tasks().count_tasks(&TaskFilter {
            topic: Some(name.clone()),
            state: TaskState::Queued,
            before_sequence: Some(task.sequence()),
        })?;

        resources.push(ResourceAvailability {
            topic,
            running,
            queued_before,
        });
    }

    Ok(resources)
}
