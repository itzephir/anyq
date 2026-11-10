use crate::{channel::Channel, client_session::ClientSession};
use anyq_daemon_api::{QueueRequest, QueueResponse};
use anyq_daemon_transport::Endpoint;
use anyq_queue_api::{Error, Result};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

pub fn status(endpoint: &Endpoint, cancelled: Arc<AtomicBool>) -> Result<Option<u32>> {
    let stream = match endpoint.connect() {
        Ok(stream) => stream,
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::NotFound | std::io::ErrorKind::ConnectionRefused
            ) =>
        {
            return Ok(None);
        }
        Err(error) => return Err(Error::Storage(error.to_string())),
    };
    let mut session = ClientSession::new(stream, cancelled);

    match session.request(QueueRequest::Ping)? {
        QueueResponse::Ready { pid, database_id } if database_id == endpoint.identity => {
            Ok(Some(pid))
        }
        _ => Err(Channel::unexpected()),
    }
}

pub fn stop(endpoint: &Endpoint, cancelled: Arc<AtomicBool>) -> Result<()> {
    let stream = match endpoint.connect() {
        Ok(stream) => stream,
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::NotFound | std::io::ErrorKind::ConnectionRefused
            ) =>
        {
            return Ok(());
        }
        Err(error) => return Err(Error::Storage(error.to_string())),
    };
    let mut session = ClientSession::new(stream, cancelled.clone());
    if !matches!(session.request(QueueRequest::Stop)?, QueueResponse::Stopped) {
        return Err(Channel::unexpected());
    }
    drop(session);

    let deadline = Instant::now() + Duration::from_secs(10);
    while endpoint.connect().is_ok() {
        if cancelled.load(Ordering::SeqCst) {
            return Err(Error::Interrupted);
        }
        if Instant::now() >= deadline {
            return Err(Error::Storage("daemon shutdown timed out".into()));
        }

        thread::sleep(Duration::from_millis(5));
    }

    Ok(())
}
