use crate::{sqlite_records::SqliteRecords, storage::storage};
use anyq_plugin_dispatch::{Records, Store};
use anyq_queue_api::{Error, Result};
use rusqlite::{Connection, ErrorCode, OpenFlags, Transaction, TransactionBehavior};
use std::{
    cell::RefCell,
    path::Path,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

#[derive(Clone)]
pub struct SqlitePluginStore {
    connection: Rc<RefCell<Connection>>,
    stop_waiting: Option<Arc<AtomicBool>>,
}

impl SqlitePluginStore {
    pub fn open(path: &Path) -> Result<Self> {
        Self::open_with_wait_policy(path, None)
    }

    pub fn open_background(path: &Path, stop_waiting: Arc<AtomicBool>) -> Result<Self> {
        Self::open_with_wait_policy(path, Some(stop_waiting))
    }

    fn open_with_wait_policy(path: &Path, stop_waiting: Option<Arc<AtomicBool>>) -> Result<Self> {
        let flags = OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX;
        let connection = Connection::open_with_flags(path, flags).map_err(storage)?;
        connection
            .busy_timeout(if stop_waiting.is_some() {
                Duration::from_millis(100)
            } else {
                Duration::from_secs(5)
            })
            .map_err(storage)?;
        connection
            .pragma_update(None, "foreign_keys", "ON")
            .map_err(storage)?;

        Ok(Self {
            connection: Rc::new(RefCell::new(connection)),
            stop_waiting,
        })
    }
}

impl Store for SqlitePluginStore {
    fn transaction<T>(
        &mut self,
        operation: impl FnOnce(&mut dyn Records) -> Result<T>,
    ) -> Result<T> {
        let connection = self.connection.borrow_mut();
        let tx = loop {
            // The exclusive RefCell borrow prevents nested connection use.
            match Transaction::new_unchecked(&connection, TransactionBehavior::Immediate) {
                Ok(tx) => break tx,
                Err(error)
                    if self.stop_waiting.is_some()
                        && matches!(
                            error.sqlite_error_code(),
                            Some(ErrorCode::DatabaseBusy | ErrorCode::FileLockingProtocolFailed)
                        ) =>
                {
                    if self.stop_waiting.as_ref().unwrap().load(Ordering::SeqCst) {
                        return Err(Error::Interrupted);
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(error) => return Err(storage(error)),
            }
        };
        let result = operation(&mut SqliteRecords { connection: &tx })?;
        tx.commit().map_err(storage)?;

        Ok(result)
    }
}
