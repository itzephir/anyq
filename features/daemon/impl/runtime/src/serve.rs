use crate::{QueueFactory, SessionSignals, session::Session, session_capacity::SessionCapacity};
use anyq_daemon_transport::{Endpoint, OwnerLock, validate_peer};
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
    cancelled: Arc<AtomicBool>,
) -> Result<()> {
    let shutdown = Arc::new(AtomicBool::new(false));
    let _owner = OwnerLock::acquire(&endpoint)
        .map_err(|error| Error::Storage(format!("queue owner lock: {error}")))?;
    // A competing opener may have created the sidecar before we locked it.
    // Publish only the spelling anchored by the lock we now own.
    let endpoint =
        Endpoint::resolve(&endpoint.database).map_err(|error| Error::Storage(error.to_string()))?;
    let listener = endpoint
        .listen()
        .map_err(|error| Error::Storage(error.to_string()))?;
    let background_factory = factory.clone();
    let background_path = endpoint.database.clone();
    let background_shutdown = shutdown.clone();
    let background = thread::spawn(move || {
        let result = background_factory.background(&background_path, background_shutdown.clone());
        if result.is_err() {
            background_shutdown.store(true, Ordering::SeqCst);
        }
        result
    });
    let mut sessions = Vec::new();
    let capacity = Arc::new(SessionCapacity::default());
    let mut failure = None;

    while !shutdown.load(Ordering::SeqCst) {
        if cancelled.load(Ordering::SeqCst) {
            shutdown.store(true, Ordering::SeqCst);
            break;
        }

        sessions.retain(
            |(signals, handle): &(Arc<SessionSignals>, thread::JoinHandle<()>)| {
                !signals.closed.load(Ordering::SeqCst) || !handle.is_finished()
            },
        );

        match listener.accept() {
            Ok(stream) => {
                if validate_peer(&stream).is_err() {
                    continue;
                }

                let Some(permit) = capacity.admit() else {
                    continue;
                };
                let signals = Arc::new(SessionSignals::default());
                let session = Session {
                    stream: Arc::new(stream),
                    endpoint: endpoint.clone(),
                    factory: factory.clone(),
                    signals: signals.clone(),
                    shutdown: shutdown.clone(),
                    permit,
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

    shutdown.store(true, Ordering::SeqCst);
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

    while !background.is_finished() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    if !background.is_finished() {
        std::process::exit(2);
    }
    let background_result = background
        .join()
        .map_err(|_| Error::Storage("plugin host stopped unexpectedly".into()))?;
    background_result?;

    failure.map_or(Ok(()), Err)
}
