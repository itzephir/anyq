use crate::{checkpoint::checkpoint, storage_error::storage};
use anyq_queue_api::Result;
use rusqlite::{Connection, Transaction, TransactionBehavior};
use std::{
    sync::atomic::AtomicBool,
    time::{Duration, Instant},
};

pub(crate) fn begin_transaction<'connection>(
    connection: &'connection Connection,
    cancelled: &AtomicBool,
    timeout: Duration,
    stop_waiting: Option<&AtomicBool>,
) -> Result<Transaction<'connection>> {
    connection
        .busy_timeout(Duration::from_millis(100))
        .map_err(storage)?;
    let deadline = Instant::now() + timeout;

    loop {
        checkpoint("sqlite.before-lock", cancelled)?;

        // The caller owns the exclusive RefCell borrow; SQLite rejects nested transactions too.
        match Transaction::new_unchecked(connection, TransactionBehavior::Immediate) {
            Ok(transaction) => {
                connection.busy_timeout(timeout).map_err(storage)?;
                return Ok(transaction);
            }
            Err(error) if crate::retry_lock::can_retry(&error, deadline, stop_waiting)? => {
                checkpoint("sqlite.waiting-lock", cancelled)?;
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(error) => return Err(storage(error)),
        }
    }
}
