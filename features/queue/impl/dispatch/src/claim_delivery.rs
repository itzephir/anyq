use crate::lookup_topic::lookup_topic;
use crate::ports::*;
use crate::recover_expired::recover_expired;
use crate::require_group::require_group;
use anyq_queue_api::*;

pub struct ClaimDeliveryInteractor<S, C, T> {
    store: S,
    clock: C,
    tokens: T,
}
impl<S, C, T> ClaimDeliveryInteractor<S, C, T> {
    pub fn new(store: S, clock: C, tokens: T) -> Self {
        Self {
            store,
            clock,
            tokens,
        }
    }
}
impl<S: Store, C: Clock, T: Tokens> ClaimDelivery for ClaimDeliveryInteractor<S, C, T> {
    fn claim(&mut self, request: Claim) -> Result<Option<Assignment>> {
        let clock = &self.clock;
        let tokens = &mut self.tokens;
        self.store.transaction(|records| {
            let now = clock.now_ms();
            validate_name(&request.consumer)?;
            deadline(now, request.lease_ms)?;
            let topic = lookup_topic(records, &request.topic)?;
            require_group(records, &topic.name, &request.group)?;
            recover_expired(records, Some(topic.name.clone()), now)?;
            let active = records.deliveries().count_deliveries(&DeliveryFilter {
                topic: Some(topic.name.clone()),
                state: Some(State::Running),
                ..Default::default()
            })?;
            if active >= u64::from(topic.capacity) {
                return Ok(None);
            }
            let ready = records.deliveries().deliveries(&DeliveryFilter {
                topic: Some(topic.name),
                group: Some(request.group),
                state: Some(State::Pending),
                ready_at: Some(now),
                limit: Some(1),
                ..Default::default()
            })?;
            let Some(mut delivery) = ready.into_iter().next() else {
                return Ok(None);
            };
            delivery.claim(request.consumer, tokens.next(), now, request.lease_ms)?;
            records.deliveries().put_delivery(&delivery)?;
            let message = records
                .messages()
                .message(delivery.message_id)?
                .ok_or_else(|| Error::Storage("delivery without message".into()))?;
            Ok(Some(Assignment { message, delivery }))
        })
    }
}
