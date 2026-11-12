use crate::{ExecutionDirectory, QueueEventKind, Task, Topic};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventSnapshot {
    pub kind: QueueEventKind,
    pub occurred_at: i64,
    pub task: Option<Task>,
    pub topics: Vec<Topic>,
    pub directory: Option<ExecutionDirectory>,
}
