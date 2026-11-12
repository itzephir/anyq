use crate::{Result, Task, TaskState};

/// Current queue fields only; metadata and execution history are not observed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedTask {
    sequence: i64,
    name: String,
    state: TaskState,
    topics: Vec<String>,
    autostart_default: bool,
    stopped: bool,
}

impl ObservedTask {
    pub fn restore(
        sequence: i64,
        name: String,
        state: TaskState,
        topics: Vec<String>,
        autostart_default: bool,
        stopped: bool,
    ) -> Result<Self> {
        Task::restore(sequence, name, state, topics)
            .map(|task| task.with_start_policy(autostart_default, stopped).into())
    }

    pub fn sequence(&self) -> i64 {
        self.sequence
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn state(&self) -> TaskState {
        self.state
    }

    pub fn topics(&self) -> &[String] {
        &self.topics
    }

    pub fn autostart_default(&self) -> bool {
        self.autostart_default
    }

    pub fn stopped(&self) -> bool {
        self.stopped
    }
}

impl From<Task> for ObservedTask {
    fn from(task: Task) -> Self {
        Self {
            sequence: task.sequence(),
            name: task.name().into(),
            state: task.state(),
            topics: task.topics().into(),
            autostart_default: task.autostart_default(),
            stopped: task.stopped(),
        }
    }
}
