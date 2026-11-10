use crate::{checkpoint::checkpoint, storage_error::storage};
use anyq_queue_api::Result;
use rusqlite::ErrorCode;
use std::{
    sync::atomic::AtomicBool,
    thread,
    time::{Duration, Instant},
};

pub(crate) fn contention(error: &rusqlite::Error) -> bool {
    matches!(
        error.sqlite_error_code(),
        Some(ErrorCode::DatabaseBusy | ErrorCode::FileLockingProtocolFailed)
    )
}

/// Retry only a failed lock acquisition or read, before any business mutation.
pub(crate) fn retry<T>(
    cancelled: &AtomicBool,
    timeout: Duration,
    mut operation: impl FnMut() -> rusqlite::Result<T>,
) -> Result<T> {
    let deadline = Instant::now() + timeout;
    loop {
        checkpoint("sqlite.before-lock", cancelled)?;
        match operation() {
            Ok(value) => return Ok(value),
            Err(error) if contention(&error) && Instant::now() < deadline => {
                thread::sleep(Duration::from_millis(10))
            }
            Err(error) => return Err(storage(error)),
        }
    }
}
