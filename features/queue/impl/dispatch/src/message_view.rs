use crate::ports::*;
use anyq_queue_api::*;

pub(crate) fn message_view(
    records: &mut dyn UnitOfWork,
    message: Message,
    group: Option<String>,
) -> Result<MessageView> {
    let deliveries = records.deliveries().deliveries(&DeliveryFilter {
        message_id: Some(message.id),
        group,
        ..Default::default()
    })?;
    Ok(MessageView {
        message,
        deliveries,
    })
}
