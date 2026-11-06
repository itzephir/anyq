use crate::ports::*;
use anyq_queue_api::*;

pub(crate) fn require_group(
    records: &mut dyn UnitOfWork,
    topic_name: &str,
    name: &str,
) -> Result<()> {
    if !records
        .groups()
        .groups(topic_name)?
        .iter()
        .any(|g| g.name == name)
    {
        return Err(Error::NotFound(format!(
            "group '{name}' does not exist in '{topic_name}'"
        )));
    }

    Ok(())
}
