use crate::lookup_topic::lookup_topic;
use crate::message_view::message_view;
use crate::ports::*;
use crate::recover_expired::recover_expired;
use crate::require_group::require_group;
use anyq_queue_api::*;

pub struct ListMessagesInteractor<S, C> {
    store: S,
    clock: C,
}

impl<S, C> ListMessagesInteractor<S, C> {
    pub fn new(store: S, clock: C) -> Self {
        Self { store, clock }
    }
}

impl<S: Store, C: Clock> ListMessages for ListMessagesInteractor<S, C> {
    fn list(&mut self, query: MessageQuery) -> Result<Vec<MessageView>> {
        let clock = &self.clock;

        self.store.transaction(|records| {
            let now = clock.now_ms();

            if query.after < 0 || !(1..=1000).contains(&query.limit) {
                return Err(Error::Invalid(
                    "after must be nonnegative; limit must be in 1..1000".into(),
                ));
            }

            if let Some(name) = &query.topic {
                lookup_topic(records, name)?;

                if let Some(group) = &query.group {
                    require_group(records, name, group)?;
                }
            }

            if query.topic.is_none() && query.group.is_some() {
                return Err(Error::Invalid("group filter requires a topic".into()));
            }

            recover_expired(records, query.topic.clone(), now)?;
            let messages =
                records
                    .messages()
                    .messages(query.topic.as_deref(), query.after, query.limit)?;
            messages
                .into_iter()
                .map(|message| message_view(records, message, query.group.clone()))
                .collect()
        })
    }
}
