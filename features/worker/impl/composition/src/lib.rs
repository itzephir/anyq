mod worker_dependencies;

use anyq_queue_api::Result;
use anyq_worker_api::{Settings, Worker};
use anyq_worker_execution::{Execution, ExecutionPorts};
use anyq_worker_process::{CommandRunner, SystemWait};
pub use worker_dependencies::WorkerDependencies;

pub fn create(
    dependencies: WorkerDependencies,
    command: Vec<String>,
    settings: Settings,
) -> Result<Box<dyn Worker>> {
    let ports = ExecutionPorts {
        claim: dependencies.claim,
        renew: dependencies.renew,
        complete: dependencies.complete,
        reject: dependencies.reject,
        topics: dependencies.topics,
    };

    Ok(Box::new(Execution::new(
        ports,
        CommandRunner::new(command)?,
        SystemWait::new()?,
        settings,
    )?))
}
