use crate::ports::*;
use anyq_queue_api::*;

pub(crate) fn recover_expired(
    records: &mut dyn UnitOfWork,
    topic_name: Option<String>,
    now: i64,
) -> Result<()> {
    let expired = records.deliveries().deliveries(&DeliveryFilter {
        topic: topic_name,
        state: Some(State::Running),
        expired_at: Some(now),
        ..Default::default()
    })?;
    for mut d in expired {
        if d.expire(now) {
            records.deliveries().put_delivery(&d)?;
        }
    }
    Ok(())
}
