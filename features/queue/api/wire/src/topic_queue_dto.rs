use crate::{TaskDto, TopicDto};
use anyq_queue_api::{Error, TopicQueue};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TopicQueueDto {
    pub topic: TopicDto,
    pub tasks: Vec<TaskDto>,
}

impl From<TopicQueue> for TopicQueueDto {
    fn from(queue: TopicQueue) -> Self {
        Self {
            topic: queue.topic.into(),
            tasks: queue.tasks.into_iter().map(Into::into).collect(),
        }
    }
}

impl TryFrom<TopicQueueDto> for TopicQueue {
    type Error = Error;

    fn try_from(value: TopicQueueDto) -> Result<Self, Error> {
        Ok(Self::new(
            value.topic.try_into()?,
            value
                .tasks
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        ))
    }
}
