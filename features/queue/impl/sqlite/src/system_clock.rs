use anyq_queue_dispatch::ports::Clock;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) struct SystemClock;

impl Clock for SystemClock {
    fn now_millis(&self) -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|time| time.as_millis().min(i64::MAX as u128) as i64)
            .unwrap_or(0)
    }
}
