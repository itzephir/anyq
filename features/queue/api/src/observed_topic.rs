use crate::{Result, Topic};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedTopic {
    pub name: String,
    pub capacity: u32,
    pub autostart: Option<bool>,
}

impl ObservedTopic {
    pub fn restore(name: String, capacity: u32, autostart: Option<bool>) -> Result<Self> {
        let mut topic = Topic::restore(name, capacity)?;
        topic.autostart = autostart;

        Ok(topic.into())
    }
}

impl From<Topic> for ObservedTopic {
    fn from(topic: Topic) -> Self {
        Self {
            name: topic.name,
            capacity: topic.capacity,
            autostart: topic.autostart,
        }
    }
}
