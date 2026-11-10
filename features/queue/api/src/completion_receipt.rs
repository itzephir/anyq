use crate::Task;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionReceipt {
    pub task: String,
    pub force: bool,
    pub result: Task,
}
