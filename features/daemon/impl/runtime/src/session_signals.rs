use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Default)]
pub struct SessionSignals {
    pub cancelled: std::sync::Arc<AtomicBool>,
    pub closed: AtomicBool,
    pub keep_alive: AtomicBool,
}

impl SessionSignals {
    pub fn close(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
        self.closed.store(true, Ordering::SeqCst);
    }
}
