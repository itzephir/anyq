use crate::{CallerDirectoryDto, TaskDto, TopicDto};
use anyq_queue_api::{Error, EventSnapshot, QueueEvent, QueueEventKind, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventDto {
    pub database_id: String,
    pub sequence: i64,
    pub version: u32,
    pub kind: String,
    pub occurred_at: i64,
    pub task: Option<TaskDto>,
    pub topics: Vec<TopicDto>,
    pub directory: Option<CallerDirectoryDto>,
}

impl TryFrom<QueueEvent> for EventDto {
    type Error = Error;

    fn try_from(event: QueueEvent) -> Result<Self> {
        Ok(Self {
            database_id: event.database_id,
            sequence: event.sequence,
            version: event.version,
            kind: event.snapshot.kind.as_str().into(),
            occurred_at: event.snapshot.occurred_at,
            task: event.snapshot.task.map(Into::into),
            topics: event.snapshot.topics.into_iter().map(Into::into).collect(),
            directory: event
                .snapshot
                .directory
                .map(TryInto::try_into)
                .transpose()?,
        })
    }
}

impl TryFrom<EventDto> for QueueEvent {
    type Error = Error;

    fn try_from(event: EventDto) -> Result<Self> {
        let kind = match event.kind.as_str() {
            "task.registered" => QueueEventKind::TaskRegistered,
            "task.linked" => QueueEventKind::TaskLinked,
            "task.running" => QueueEventKind::TaskRunning,
            "task.completed" => QueueEventKind::TaskCompleted,
            "task.metadata-changed" => QueueEventKind::TaskMetadataChanged,
            "topic.created" => QueueEventKind::TopicCreated,
            "topic.capacity-changed" => QueueEventKind::TopicCapacityChanged,
            "topic.autostart-changed" => QueueEventKind::TopicAutostartChanged,
            "topic.metadata-changed" => QueueEventKind::TopicMetadataChanged,
            _ => return Err(Error::Storage("unknown queue event kind".into())),
        };
        if event.version != 1 || event.sequence <= 0 || event.database_id.is_empty() {
            return Err(Error::Storage("invalid queue event identity".into()));
        }

        Ok(Self {
            database_id: event.database_id,
            sequence: event.sequence,
            version: event.version,
            snapshot: EventSnapshot {
                kind,
                occurred_at: event.occurred_at,
                task: event.task.map(TryInto::try_into).transpose()?,
                topics: event
                    .topics
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_>>()?,
                directory: event.directory.map(Into::into),
            },
        })
    }
}
