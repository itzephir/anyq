use crate::{QueueFactory, SessionSignals, session::Session};
use anyq_daemon_transport::{Endpoint, OwnerLock, traits::Listener, validate_peer};
use anyq_queue_api::{Error, Result};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

pub fn serve(
    endpoint: Endpoint,
    factory: Arc<dyn QueueFactory>,
    shutdown: Arc<AtomicBool>,
) -> Result<()> {
    let _owner = OwnerLock::acquire(&endpoint)
        .map_err(|error| Error::Storage(format!("queue owner lock: {error}")))?;
    let listener = endpoint
        .listen()
        .map_err(|error| Error::Storage(error.to_string()))?;
    let mut sessions = Vec::new();
    let mut failure = None;

    while !shutdown.load(Ordering::SeqCst) {
        sessions.retain(
            |(signals, handle): &(Arc<SessionSignals>, thread::JoinHandle<()>)| {
                !signals.closed.load(Ordering::SeqCst) || !handle.is_finished()
            },
        );

        match listener.accept() {
            Ok(stream) => {
                if sessions.len() >= 64 || validate_peer(&stream).is_err() {
                    continue;
                }

                let signals = Arc::new(SessionSignals::default());
                let session = Session {
                    stream: Arc::new(stream),
                    endpoint: endpoint.clone(),
                    factory: factory.clone(),
                    signals: signals.clone(),
                    shutdown: shutdown.clone(),
                };
                sessions.push((signals, thread::spawn(move || session.run())));
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(2))
            }
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
            Err(error) => {
                failure = Some(Error::Storage(error.to_string()));
                break;
            }
        }
    }

    drop(listener);
    for (signals, _) in &sessions {
        signals.close();
    }

    let deadline = Instant::now() + Duration::from_secs(10);
    for (_, handle) in sessions {
        while !handle.is_finished() && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(2));
        }

        if !handle.is_finished() {
            // Returning would drop the lifetime lock while detached workers may
            // still write. Process termination releases the lock and workers
            // together; SQLite recovers any uncommitted transaction.
            std::process::exit(2);
        }

        let _ = handle.join();
    }

    failure.map_or(Ok(()), Err)
}
