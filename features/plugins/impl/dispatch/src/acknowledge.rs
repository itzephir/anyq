use crate::Store;
use anyq_plugin_api::{AcknowledgeRequest, MAX_TIMERS, TimerUpdate};
use anyq_queue_api::{Error, Result};

pub fn acknowledge(store: &mut impl Store, request: AcknowledgeRequest) -> Result<()> {
    if request.timers.len() > MAX_TIMERS {
        return Err(Error::Invalid("too many timer updates".into()));
    }
    for timer in &request.timers {
        let id = match timer {
            TimerUpdate::Put(timer) => &timer.id,
            TimerUpdate::Remove(id) => id,
        };
        if id.is_empty() || id.len() > 512 {
            return Err(Error::Invalid("invalid timer ID".into()));
        }
    }

    store.transaction(|records| {
        let mut plugin = records
            .registration(&request.plugin)?
            .ok_or_else(|| Error::NotFound("plugin is absent".into()))?;
        if !plugin.enabled {
            return Err(Error::Conflict("plugin is disabled".into()));
        }
        if request
            .sequence
            .is_some_and(|sequence| sequence <= plugin.cursor)
        {
            return Ok(());
        }
        if let Some((id, deadline)) = &request.timer {
            let pending = records
                .timers(&plugin.name)?
                .into_iter()
                .find(|timer| &timer.id == id && timer.deadline == *deadline);
            if pending.is_none() {
                return Ok(());
            }
            records.remove_timer(&plugin.name, id)?;
        }
        if let Some(sequence) = request.sequence {
            plugin.cursor = sequence;
        }
        if let Some(state) = request.state {
            plugin.state = state;
        }
        for update in &request.timers {
            match update {
                TimerUpdate::Put(timer) => records.put_timer(&plugin.name, timer)?,
                TimerUpdate::Remove(id) => records.remove_timer(&plugin.name, id)?,
            }
        }
        if records.timers(&plugin.name)?.len() > MAX_TIMERS {
            return Err(Error::Invalid("plugin timer limit exceeded".into()));
        }
        plugin.failures = 0;
        plugin.last_failure = None;
        records.save(&plugin)
    })
}
