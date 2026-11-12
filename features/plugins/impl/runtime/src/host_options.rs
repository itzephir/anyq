use std::time::Duration;

#[derive(Debug, Clone)]
pub struct HostOptions {
    pub response_timeout: Duration,
    pub cancellation_timeout: Duration,
    pub poll_interval: Duration,
    pub retained_events: i64,
}

impl Default for HostOptions {
    fn default() -> Self {
        Self {
            response_timeout: Duration::from_secs(5),
            cancellation_timeout: Duration::from_secs(3),
            poll_interval: Duration::from_millis(10),
            retained_events: 10_000,
        }
    }
}
