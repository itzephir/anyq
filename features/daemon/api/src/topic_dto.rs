use anyq_queue_api::{Error, Topic};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TopicDto {
    pub name: String,
    pub capacity: u32,
    pub autostart: Option<bool>,
}

impl From<Topic> for TopicDto {
    fn from(topic: Topic) -> Self {
        Self {
            name: topic.name,
            capacity: topic.capacity,
            autostart: topic.autostart,
        }
    }
}

impl TryFrom<TopicDto> for Topic {
    type Error = Error;

    fn try_from(value: TopicDto) -> Result<Self, Error> {
        let mut topic = Self::restore(value.name, value.capacity)?;
        topic.autostart = value.autostart;

        Ok(topic)
    }
}
