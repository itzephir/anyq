#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManagedWorkState {
    LaunchPending,
    Active,
    Stopped,
    Uncertain,
}

impl ManagedWorkState {
    pub const BLOCKING_STATES: [Self; 3] = [Self::LaunchPending, Self::Active, Self::Uncertain];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::LaunchPending => "launch-pending",
            Self::Active => "active",
            Self::Stopped => "stopped",
            Self::Uncertain => "uncertain",
        }
    }

    pub fn blocks_completion(self) -> bool {
        Self::BLOCKING_STATES.contains(&self)
    }
}
