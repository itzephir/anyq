use anyq_queue_api::{Queue, Result};
use anyq_worker_api::{Settings, Worker};
use anyq_worker_execution::Execution;
use anyq_worker_process::{CommandRunner, SystemWait};

pub fn create(
    queue: Box<dyn Queue>,
    command: Vec<String>,
    settings: Settings,
) -> Result<Box<dyn Worker>> {
    Ok(Box::new(Execution::new(
        queue,
        CommandRunner::new(command)?,
        SystemWait::new()?,
        settings,
    )?))
}
