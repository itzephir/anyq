use crate::ports::*;
use anyq_queue_api::*;

pub struct CreateTopicInteractor<S> {
    store: S,
}

impl<S> CreateTopicInteractor<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S: Store> CreateTopic for CreateTopicInteractor<S> {
    fn create(&mut self, topic: Topic) -> Result<Topic> {
        self.store.transaction(|records| {
            let mut validated = Topic::restore(topic.name, topic.capacity)?;
            validated.autostart = topic.autostart;
            validated.metadata = topic.metadata;
            let topic = validated;

            if let Some(existing) = records.topics().topic(&topic.name)? {
                if existing.capacity != topic.capacity {
                    return Err(Error::Conflict(
                        "topic exists with another capacity; use topic capacity".into(),
                    ));
                }

                return Ok(existing);
            } else {
                records.topics().put_topic(&topic)?;
            }

            Ok(topic)
        })
    }
}
