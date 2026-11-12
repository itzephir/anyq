use anyq_plugin_api::PluginStatus;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PluginStatusDto {
    pub name: String,
    pub enabled: bool,
    pub running: bool,
    pub cursor: i64,
    pub lag: i64,
    pub failures: u32,
    pub last_failure: Option<String>,
    pub active_timers: u64,
    pub unresolved_actions: u64,
}

impl From<PluginStatus> for PluginStatusDto {
    fn from(status: PluginStatus) -> Self {
        Self {
            name: status.name,
            enabled: status.enabled,
            running: status.running,
            cursor: status.cursor,
            lag: status.lag,
            failures: status.failures,
            last_failure: status.last_failure,
            active_timers: status.active_timers,
            unresolved_actions: status.unresolved_actions,
        }
    }
}

impl From<PluginStatusDto> for PluginStatus {
    fn from(status: PluginStatusDto) -> Self {
        Self {
            name: status.name,
            enabled: status.enabled,
            running: status.running,
            cursor: status.cursor,
            lag: status.lag,
            failures: status.failures,
            last_failure: status.last_failure,
            active_timers: status.active_timers,
            unresolved_actions: status.unresolved_actions,
        }
    }
}
