use crate::{Task, TaskState, Topic};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopicQueue {
    pub topic: Topic,
    pub tasks: Vec<Task>,
}

impl TopicQueue {
    pub const STATES: [TaskState; 2] = [TaskState::Queued, TaskState::Running];

    pub fn new(topic: Topic, mut tasks: Vec<Task>) -> Self {
        tasks.sort_by_key(Task::sequence);

        Self { topic, tasks }
    }
}
