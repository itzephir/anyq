use anyq_queue_api::{Error, Result, TaskState};

pub(crate) fn decode(value: &str) -> Result<TaskState> {
    match value {
        "registered" => Ok(TaskState::Registered),
        "queued" => Ok(TaskState::Queued),
        "running" => Ok(TaskState::Running),
        "completed" => Ok(TaskState::Completed),
        _ => Err(Error::Storage("invalid daemon task state".into())),
    }
}
