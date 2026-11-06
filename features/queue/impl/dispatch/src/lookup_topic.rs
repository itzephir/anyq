use crate::ports::*;
use anyq_queue_api::*;

pub(crate) fn lookup_topic(records: &mut dyn UnitOfWork, name: &str) -> Result<Topic> {
    records
        .topics()
        .topic(name)?
        .ok_or_else(|| Error::NotFound(format!("topic '{name}' does not exist")))
}
