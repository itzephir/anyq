use anyq_plugin_runtime::Clock;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) struct WallClock;

impl Clock for WallClock {
    fn now_millis(&self) -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|time| time.as_millis().min(i64::MAX as u128) as i64)
            .unwrap_or(0)
    }
}
