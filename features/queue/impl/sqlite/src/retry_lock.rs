use crate::{checkpoint::checkpoint, storage_error::storage};
use anyq_queue_api::{Error, Result};
use rusqlite::ErrorCode;
use std::{
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant},
};

pub(crate) fn contention(error: &rusqlite::Error) -> bool {
    matches!(
        error.sqlite_error_code(),
        Some(ErrorCode::DatabaseBusy | ErrorCode::FileLockingProtocolFailed)
    )
}

pub(crate) fn can_retry(
    error: &rusqlite::Error,
    deadline: Instant,
    stop_waiting: Option<&AtomicBool>,
) -> Result<bool> {
    if !contention(error) {
        return Ok(false);
    }

    if let Some(stopping) = stop_waiting {
        if stopping.load(Ordering::SeqCst) {
            return Err(Error::Interrupted);
        }

        return Ok(true);
    }

    Ok(Instant::now() < deadline)
}

/// Retry only a failed lock acquisition or read, before any business mutation.
pub(crate) fn retry<T>(
    cancelled: &AtomicBool,
    timeout: Duration,
    stop_waiting: Option<&AtomicBool>,
    mut operation: impl FnMut() -> rusqlite::Result<T>,
) -> Result<T> {
    let deadline = Instant::now() + timeout;
    loop {
        checkpoint("sqlite.before-lock", cancelled)?;
        match operation() {
            Ok(value) => return Ok(value),
            Err(error) if can_retry(&error, deadline, stop_waiting)? => {
                thread::sleep(Duration::from_millis(10))
            }
            Err(error) => return Err(storage(error)),
        }
    }
}
