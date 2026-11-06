use crate::ports::*;
use anyq_queue_api::*;

pub(crate) fn lookup_delivery(
    records: &mut dyn UnitOfWork,
    id: i64,
    group: String,
) -> Result<Delivery> {
    records
        .deliveries()
        .deliveries(&DeliveryFilter {
            message_id: Some(id),
            group: Some(group),
            ..Default::default()
        })?
        .into_iter()
        .next()
        .ok_or_else(|| Error::NotFound("delivery does not exist".into()))
}
