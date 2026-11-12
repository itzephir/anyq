use crate::ManagedWork;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedLaunch {
    pub work: ManagedWork,
    pub launch_allowed: bool,
}
