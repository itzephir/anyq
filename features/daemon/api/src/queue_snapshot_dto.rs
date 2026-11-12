use crate::{ObservedTaskDto, ObservedTopicDto};
use anyq_queue_api::{QueueSnapshot, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueueSnapshotDto {
    pub topics: Vec<ObservedTopicDto>,
    pub tasks: Vec<ObservedTaskDto>,
}

impl From<QueueSnapshot> for QueueSnapshotDto {
    fn from(snapshot: QueueSnapshot) -> Self {
        Self {
            topics: snapshot.topics.into_iter().map(Into::into).collect(),
            tasks: snapshot.tasks.into_iter().map(Into::into).collect(),
        }
    }
}

impl TryFrom<QueueSnapshotDto> for QueueSnapshot {
    type Error = anyq_queue_api::Error;

    fn try_from(dto: QueueSnapshotDto) -> Result<Self> {
        Self::new(
            dto.topics
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_>>()?,
            dto.tasks
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_>>()?,
        )
    }
}
