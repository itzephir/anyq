use crate::AutoCreatePolicy;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QueuePolicy {
    pub autocreate: AutoCreatePolicy,
    pub task_autostart: bool,
}

impl Default for QueuePolicy {
    fn default() -> Self {
        Self {
            autocreate: AutoCreatePolicy::default(),
            task_autostart: true,
        }
    }
}
