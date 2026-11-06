#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Pending,
    Running,
    Succeeded,
    Failed,
}
impl State {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
        }
    }
}
