use anyq_plugin_api::PluginStatus;
use serde_json::{Value, json};

pub fn status(status: PluginStatus) -> Value {
    json!({
        "name": status.name,
        "enabled": status.enabled,
        "running": status.running,
        "cursor": status.cursor,
        "lag": status.lag,
        "failures": status.failures,
        "last_failure": status.last_failure,
        "active_timers": status.active_timers,
        "unresolved_actions": status.unresolved_actions,
    })
}
