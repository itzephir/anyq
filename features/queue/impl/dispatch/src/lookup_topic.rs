use crate::ports::UnitOfWork;
use anyq_queue_api::{AutoCreatePolicy, Result, Topic, parse_topic_name};

pub(crate) fn lookup_topic(
    records: &mut dyn UnitOfWork,
    input: &str,
    policy: AutoCreatePolicy,
) -> Result<Topic> {
    lookup_topic_value(records, &parse_topic_name(input)?, policy)
}

pub(crate) fn lookup_topic_value(
    records: &mut dyn UnitOfWork,
    name: &str,
    policy: AutoCreatePolicy,
) -> Result<Topic> {
    if let Some(topic) = records.topics().topic(name)? {
        return Ok(topic);
    }

    policy.require_topic_creation(name)?;
    let topic = Topic::restore(name.into(), Topic::DEFAULT_CAPACITY)?;
    records.topics().put_topic(&topic)?;

    Ok(topic)
}
