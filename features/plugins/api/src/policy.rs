use anyq_queue_api::{Error, Result};

pub const MAX_TIMERS: usize = 128;
pub const CAPABILITIES: [&str; 5] = ["read", "complete", "state", "timers", "managed"];
pub const EVENT_KINDS: [&str; 9] = [
    "task.registered",
    "task.linked",
    "task.running",
    "task.completed",
    "task.metadata-changed",
    "topic.created",
    "topic.capacity-changed",
    "topic.autostart-changed",
    "topic.metadata-changed",
];

pub fn validate_plugin_name(name: &str) -> Result<()> {
    if name.is_empty()
        || name.len() > 128
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(Error::Invalid(
            "plugin name must contain 1..128 ASCII letters, digits, dots, hyphens or underscores"
                .into(),
        ));
    }

    Ok(())
}

pub fn validate_handshake(subscriptions: &[String], capabilities: &[String]) -> Result<()> {
    if subscriptions.len() > EVENT_KINDS.len()
        || capabilities.len() > CAPABILITIES.len()
        || subscriptions
            .iter()
            .any(|kind| !EVENT_KINDS.contains(&kind.as_str()))
        || capabilities
            .iter()
            .any(|capability| !CAPABILITIES.contains(&capability.as_str()))
    {
        return Err(Error::Invalid(
            "unsupported plugin subscriptions or capabilities".into(),
        ));
    }

    Ok(())
}

pub fn retry_delay(failures: u32) -> i64 {
    250_i64
        .saturating_mul(1_i64 << failures.saturating_sub(1).min(8))
        .min(60_000)
}
