#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueueEventKind {
    TaskRegistered,
    TaskLinked,
    TaskRunning,
    TaskCompleted,
    TaskMetadataChanged,
    TopicCreated,
    TopicCapacityChanged,
    TopicAutostartChanged,
    TopicMetadataChanged,
}

impl QueueEventKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TaskRegistered => "task.registered",
            Self::TaskLinked => "task.linked",
            Self::TaskRunning => "task.running",
            Self::TaskCompleted => "task.completed",
            Self::TaskMetadataChanged => "task.metadata-changed",
            Self::TopicCreated => "topic.created",
            Self::TopicCapacityChanged => "topic.capacity-changed",
            Self::TopicAutostartChanged => "topic.autostart-changed",
            Self::TopicMetadataChanged => "topic.metadata-changed",
        }
    }
}
