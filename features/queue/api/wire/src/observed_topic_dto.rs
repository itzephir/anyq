use anyq_queue_api::{Error, ObservedTopic};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedTopicDto {
    pub name: String,
    pub capacity: u32,
    pub autostart: Option<bool>,
}

impl From<ObservedTopic> for ObservedTopicDto {
    fn from(topic: ObservedTopic) -> Self {
        Self {
            name: topic.name,
            capacity: topic.capacity,
            autostart: topic.autostart,
        }
    }
}

impl TryFrom<ObservedTopicDto> for ObservedTopic {
    type Error = Error;

    fn try_from(topic: ObservedTopicDto) -> Result<Self, Error> {
        Self::restore(topic.name, topic.capacity, topic.autostart)
    }
}
