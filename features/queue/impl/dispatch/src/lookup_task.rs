use crate::ports::UnitOfWork;
use anyq_queue_api::{Error, Result, Task};

pub(crate) fn lookup_task(records: &mut dyn UnitOfWork, name: &str) -> Result<Task> {
    records
        .tasks()
        .task(name)?
        .ok_or_else(|| Error::NotFound(format!("task '{name}' does not exist")))
}
