use crate::{Error, ManagedWorkState, Result, TaskState};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedWork {
    pub owner: String,
    pub action_id: String,
    pub task: String,
    pub state: ManagedWorkState,
    pub cancellation_requested: bool,
}

impl ManagedWork {
    pub fn require_launch(state: TaskState, completion_requested: bool) -> Result<()> {
        if state != TaskState::Running || completion_requested {
            return Err(Error::Conflict(
                "managed launch requires a running task without completion intent".into(),
            ));
        }

        Ok(())
    }

    pub fn report(&mut self, state: ManagedWorkState) -> Result<()> {
        if state == ManagedWorkState::LaunchPending
            || self.state == ManagedWorkState::Stopped && state != ManagedWorkState::Stopped
        {
            return Err(Error::Conflict(
                "managed action cannot be relaunched".into(),
            ));
        }
        if self.state == ManagedWorkState::Uncertain && state == ManagedWorkState::Active {
            return Err(Error::Conflict(
                "uncertain execution requires confirmed cleanup".into(),
            ));
        }
        self.state = state;

        Ok(())
    }
}

pub fn validate_action_identity(owner: &str, action_id: &str) -> Result<()> {
    if owner.is_empty()
        || owner.len() > 128
        || action_id.is_empty()
        || action_id.len() > 512
        || owner.contains('\0')
        || action_id.contains('\0')
    {
        return Err(Error::Invalid("invalid managed owner or action ID".into()));
    }

    Ok(())
}
