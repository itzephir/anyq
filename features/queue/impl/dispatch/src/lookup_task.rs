use crate::ports::UnitOfWork;
use anyq_queue_api::{AutoCreatePolicy, Result, Task, parse_name};

pub(crate) fn lookup_task(
    records: &mut dyn UnitOfWork,
    input: &str,
    policy: AutoCreatePolicy,
) -> Result<Task> {
    let name = parse_name(input)?;

    if let Some(task) = records.tasks().task(&name)? {
        return Ok(task);
    }

    policy.require_task_creation(&name)?;
    records.tasks().insert_task(&name)
}
