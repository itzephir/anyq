use crate::Records;
use anyq_plugin_api::PluginStatus;
use anyq_queue_api::{Error, ManagedWorkState, Result};

pub(crate) fn read_status(records: &mut dyn Records, name: &str) -> Result<PluginStatus> {
    let plugin = records
        .registration(name)?
        .ok_or_else(|| Error::NotFound("plugin is not registered".into()))?;
    let watermark = records.watermark()?;
    let active_timers = records.timers(name)?.len() as u64;
    let unresolved_actions = records.count_managed(name, &ManagedWorkState::BLOCKING_STATES)?;

    Ok(PluginStatus {
        state: plugin.state,
        name: plugin.name,
        enabled: plugin.enabled,
        running: plugin.running,
        cursor: plugin.cursor,
        lag: watermark.saturating_sub(plugin.cursor).max(0),
        failures: plugin.failures,
        last_failure: plugin.last_failure,
        active_timers,
        unresolved_actions,
    })
}
