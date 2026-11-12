use crate::{PluginProcess, pending_delivery::PendingDelivery};
use std::time::Instant;

pub(crate) struct PluginSession {
    pub process: Box<dyn PluginProcess>,
    pub handshake: bool,
    pub prefer_timer: bool,
    pub started_at: Instant,
    pub pending: Option<PendingDelivery>,
    pub last_cancellation: Instant,
    pub cancellation_cursor: usize,
    pub sent_cancellations: std::collections::BTreeSet<String>,
    pub draining_since: Option<Instant>,
}
