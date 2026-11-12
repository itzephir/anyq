use anyq_queue_api::{Error, ManagedWork, ManagedWorkState, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManagedWorkDto {
    pub details: anyq_queue_wire::MetadataDto,
    pub owner: String,
    pub action_id: String,
    pub task: String,
    pub state: String,
    pub cancellation_requested: bool,
}

impl From<ManagedWork> for ManagedWorkDto {
    fn from(work: ManagedWork) -> Self {
        Self {
            details: work.details.into(),
            owner: work.owner,
            action_id: work.action_id,
            task: work.task,
            state: work.state.as_str().into(),
            cancellation_requested: work.cancellation_requested,
        }
    }
}

pub fn work_state(value: &str) -> Result<ManagedWorkState> {
    match value {
        "active" => Ok(ManagedWorkState::Active),
        "stopped" => Ok(ManagedWorkState::Stopped),
        "uncertain" => Ok(ManagedWorkState::Uncertain),
        _ => Err(Error::Invalid("unsupported managed work report".into())),
    }
}
