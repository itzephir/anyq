use crate::{Error, ManagedWork, ManagedWorkState, Result, validate_action_identity};

#[derive(Debug, Default, Clone)]
pub struct ManagedWorkFilter {
    pub owner: Option<String>,
    pub task: Option<String>,
    pub action: Option<String>,
    pub after: Option<String>,
    pub limit: Option<usize>,
    pub states: Vec<ManagedWorkState>,
}

impl ManagedWorkFilter {
    pub fn validate(&self) -> Result<()> {
        if self.limit.is_some_and(|limit| limit == 0 || limit > 256) || self.states.len() > 4 {
            return Err(Error::Invalid("invalid managed-work page bound".into()));
        }
        if self.after.is_some() && (self.owner.is_none() || self.action.is_some()) {
            return Err(Error::Invalid(
                "managed-work cursor requires one owner".into(),
            ));
        }
        if let Some(action) = self.action.as_deref().or(self.after.as_deref()) {
            validate_action_identity(self.owner.as_deref().unwrap_or("query"), action)?;
        }

        Ok(())
    }

    pub fn matches(&self, work: &ManagedWork) -> bool {
        self.owner
            .as_deref()
            .is_none_or(|owner| owner == work.owner)
            && self.task.as_deref().is_none_or(|task| task == work.task)
            && self
                .action
                .as_deref()
                .is_none_or(|action| action == work.action_id)
            && self
                .after
                .as_deref()
                .is_none_or(|after| work.action_id.as_str() > after)
            && (self.states.is_empty() || self.states.contains(&work.state))
    }
}
