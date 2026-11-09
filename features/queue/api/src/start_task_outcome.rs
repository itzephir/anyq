#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartTaskOutcome {
    Started,
    AlreadyRunning,
    Busy,
}
