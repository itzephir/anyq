#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Run {
    Idle,
    Succeeded(i64),
    Failed(i64),
    Stopped,
}
