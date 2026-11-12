use anyq_queue_api::{Error, ObservedTask};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedTaskDto {
    pub sequence: i64,
    pub name: String,
    pub state: String,
    pub topics: Vec<String>,
    pub autostart_default: bool,
    pub stopped: bool,
}

impl From<ObservedTask> for ObservedTaskDto {
    fn from(task: ObservedTask) -> Self {
        Self {
            sequence: task.sequence(),
            name: task.name().into(),
            state: task.state().as_str().into(),
            topics: task.topics().into(),
            autostart_default: task.autostart_default(),
            stopped: task.stopped(),
        }
    }
}

impl TryFrom<ObservedTaskDto> for ObservedTask {
    type Error = Error;

    fn try_from(task: ObservedTaskDto) -> Result<Self, Error> {
        Self::restore(
            task.sequence,
            task.name,
            crate::decode_task_state::decode(&task.state)?,
            task.topics,
            task.autostart_default,
            task.stopped,
        )
    }
}
