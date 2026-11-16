use std::{
    io,
    sync::Arc,
    task::{Context, Poll, Wake, Waker},
    thread::{self, Thread},
    time::Duration,
};

struct IoWake(Thread);

impl Wake for IoWake {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.0.unpark();
    }
}

pub(crate) fn poll_io<T>(
    mut operation: impl FnMut(&mut Context<'_>) -> Poll<io::Result<T>>,
) -> io::Result<T> {
    let waker = Waker::from(Arc::new(IoWake(thread::current())));
    let mut context = Context::from_waker(&waker);

    if let Poll::Ready(result) = operation(&mut context) {
        return result;
    }

    // Readiness wakes the caller immediately even when macOS coalesces background
    // timers. Bound an idle wait so framing can still check cancellation/deadlines.
    thread::park_timeout(Duration::from_millis(2));

    match operation(&mut context) {
        Poll::Ready(result) => result,
        Poll::Pending => Err(io::ErrorKind::WouldBlock.into()),
    }
}
