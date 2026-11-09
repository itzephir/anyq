#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Registered,
    Queued,
    Running,
    Completed,
}

impl TaskState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Registered => "registered",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Completed => "completed",
        }
    }
}
