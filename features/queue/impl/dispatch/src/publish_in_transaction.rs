use crate::lookup_topic::lookup_topic;
use crate::ports::*;
use anyq_queue_api::*;

pub(crate) fn publish_in_transaction(
    records: &mut dyn UnitOfWork,
    request: &Publish,
    now: i64,
) -> Result<Message> {
    request.validate()?;
    lookup_topic(records, &request.topic)?;
    if let Some(key) = &request.key {
        if let Some(existing) = records.messages().message_by_key(&request.topic, key)? {
            return Ok(existing);
        }
    }
    let message = records.messages().insert_message(request, now)?;
    for g in records.groups().groups(&message.topic)? {
        records
            .deliveries()
            .put_delivery(&Delivery::pending(&message, g.name))?;
    }
    Ok(message)
}
