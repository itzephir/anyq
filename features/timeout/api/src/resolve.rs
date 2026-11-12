use crate::{Deadline, Duration};
use anyq_queue_api::{Error, MetadataValue, QueueEvent, Result};

pub fn resolve(event: &QueueEvent) -> Result<Option<Deadline>> {
    let task = event
        .snapshot
        .task
        .as_ref()
        .ok_or_else(|| Error::Invalid("timeout event has no task".into()))?;
    let sources = std::iter::once((format!("task:{}", task.name()), task.metadata())).chain(
        event
            .snapshot
            .topics
            .iter()
            .map(|topic| (format!("topic:{}", topic.name), &topic.metadata)),
    );
    let mut selected: Option<Deadline> = None;

    for (source, metadata) in sources {
        let MetadataValue::Object(object) = metadata.value() else {
            unreachable!()
        };
        let Some(value) = object.get("timeout") else {
            continue;
        };
        let MetadataValue::String(text) = value else {
            return Err(Error::Invalid(format!(
                "invalid timeout at {source}: expected duration string; value={value:?}"
            )));
        };
        let deadline = Duration::parse(text)
            .and_then(|value| value.deadline(event.snapshot.occurred_at))
            .map_err(|error| {
                Error::Invalid(format!(
                    "invalid timeout at {source}: value={text:?}; {error}"
                ))
            })?;
        let candidate = Deadline {
            event_id: event.id(),
            task: task.name().into(),
            deadline,
            source,
            value: text.clone(),
        };
        if selected
            .as_ref()
            .is_none_or(|old| candidate.deadline < old.deadline)
        {
            selected = Some(candidate);
        }
    }

    Ok(selected)
}
