use std::{
    io,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

pub(crate) struct ReceiveDeadline<'a> {
    expires: Instant,
    timeout: Duration,
    active: Option<&'a AtomicBool>,
    started: bool,
}

impl<'a> ReceiveDeadline<'a> {
    pub fn new(timeout: Duration, active: Option<&'a AtomicBool>, started: bool) -> Self {
        Self {
            expires: Instant::now() + timeout,
            timeout,
            active,
            started,
        }
    }

    pub fn check(&mut self) -> io::Result<()> {
        if !self.started
            && self
                .active
                .is_some_and(|active| active.load(Ordering::SeqCst))
        {
            self.expires = Instant::now() + self.timeout;
        }

        if Instant::now() >= self.expires {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "daemon receive timed out",
            ));
        }

        Ok(())
    }

    pub fn progress(&mut self) {
        if !self.started && self.active.is_some() {
            self.expires = Instant::now() + self.timeout;
        }

        self.started = true;
    }
}
