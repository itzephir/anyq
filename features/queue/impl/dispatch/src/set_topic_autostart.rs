use crate::record_event::topic_event;
use crate::{admit_queued_tasks::admit_queued_tasks, lookup_topic::lookup_topic, ports::Store};
use anyq_queue_api::QueueEventKind;
use anyq_queue_api::{AutoCreatePolicy, Result, SetTopicAutostart, Topic};

pub struct SetTopicAutostartInteractor<S> {
    store: S,
    policy: AutoCreatePolicy,
}

impl<S> SetTopicAutostartInteractor<S> {
    pub fn new(store: S) -> Self {
        Self::with_policy(store, AutoCreatePolicy::default())
    }

    pub fn with_policy(store: S, policy: AutoCreatePolicy) -> Self {
        Self { store, policy }
    }
}

impl<S: Store> SetTopicAutostart for SetTopicAutostartInteractor<S> {
    fn set_autostart(&mut self, name: &str, enabled: bool) -> Result<Topic> {
        self.store.transaction(|records| {
            let mut topic = lookup_topic(records, name, self.policy)?;
            let previous = topic.autostart;
            topic.autostart = Some(enabled);
            records.topics().put_topic(&topic)?;

            if previous != topic.autostart {
                topic_event(records, QueueEventKind::TopicAutostartChanged, &topic)?;
            }
            admit_queued_tasks(records)?;

            Ok(topic)
        })
    }
}
