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
            let topic = Topic::restore(topic.name, topic.capacity)?;

            if let Some(existing) = records.topics().topic(&topic.name)? {
                if existing != topic {
                    return Err(Error::Conflict(
                        "topic exists with another capacity; use topic capacity".into(),
                    ));
                }
            } else {
                records.topics().put_topic(&topic)?;
            }

            Ok(topic)
        })
    }
}
