use crate::MetadataDto;
use anyq_queue_api::{Error, Task, TaskState};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskDto {
    pub sequence: i64,
    pub name: String,
    pub state: String,
    pub topics: Vec<String>,
    pub autostart_default: bool,
    pub stopped: bool,
    pub metadata: MetadataDto,
}

impl From<Task> for TaskDto {
    fn from(task: Task) -> Self {
        Self {
            sequence: task.sequence(),
            name: task.name().into(),
            state: task.state().as_str().into(),
            topics: task.topics().to_vec(),
            autostart_default: task.autostart_default(),
            stopped: task.stopped(),
            metadata: task.metadata().clone().into(),
        }
    }
}

impl TryFrom<TaskDto> for Task {
    type Error = Error;

    fn try_from(value: TaskDto) -> Result<Self, Error> {
        let state = match value.state.as_str() {
            "registered" => TaskState::Registered,
            "queued" => TaskState::Queued,
            "running" => TaskState::Running,
            "completed" => TaskState::Completed,
            _ => return Err(Error::Storage("invalid daemon task state".into())),
        };

        Self::restore_with_metadata(
            value.sequence,
            value.name,
            state,
            value.topics,
            value.metadata.try_into()?,
        )
        .map(|task| task.with_start_policy(value.autostart_default, value.stopped))
    }
}
