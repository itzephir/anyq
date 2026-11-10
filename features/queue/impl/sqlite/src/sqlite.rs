use crate::begin_transaction::begin_transaction;
use crate::checkpoint::checkpoint;
use crate::system_clock::SystemClock;
use crate::{sqlite_unit_of_work::SqliteUnitOfWork, storage_error::storage};
use anyq_queue_api::{Error, ExecutionDirectory, Result};
use anyq_queue_dispatch::ports::Clock;
use anyq_queue_dispatch::ports::{Store, UnitOfWork};
use rusqlite::{Connection, ErrorCode, TransactionBehavior};
use std::{
    cell::RefCell,
    path::Path,
    rc::Rc,
    sync::{Arc, atomic::AtomicBool},
    thread,
    time::{Duration, Instant},
};

const LOCK_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone)]
pub struct Sqlite {
    pub(crate) connection: Rc<RefCell<Connection>>,
    cancelled: Arc<AtomicBool>,
    clock: Rc<dyn Clock>,
    directory: Option<ExecutionDirectory>,
}

fn compatible_version(connection: &Connection) -> Result<i64> {
    let version: i64 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(storage)?;

    if ![0, 2, 3, 4, 5].contains(&version) {
        return Err(Error::Storage(format!(
            "unsupported database schema version {version}; choose a separate database with --db"
        )));
    }

    if version == 0 {
        let tables: i64 = connection.query_row(
            "SELECT count(*) FROM sqlite_schema WHERE type='table' AND name NOT LIKE 'sqlite_%'",
            [], |row| row.get(0),
        ).map_err(storage)?;

        if tables != 0 {
            return Err(Error::Storage(
                "unversioned nonempty database; choose a separate database with --db".into(),
            ));
        }
    }

    Ok(version)
}

fn enable_wal(connection: &Connection, cancelled: &AtomicBool) -> Result<()> {
    let deadline = Instant::now() + LOCK_TIMEOUT;

    loop {
        checkpoint("sqlite.before-wal", cancelled)?;
        match connection.pragma_update(None, "journal_mode", "WAL") {
            Ok(()) => return Ok(()),
            Err(error)
                if error.sqlite_error_code() == Some(ErrorCode::DatabaseBusy)
                    && Instant::now() < deadline =>
            {
                // Journal-mode lock upgrades can return BUSY without calling the busy handler.
                thread::sleep(Duration::from_millis(10));
            }
            Err(error) => return Err(storage(error)),
        }
    }
}

impl Sqlite {
    pub fn with_clock(mut self, clock: Rc<dyn Clock>) -> Self {
        self.clock = clock;

        self
    }

    pub fn with_directory(mut self, directory: ExecutionDirectory) -> Self {
        self.directory = Some(directory);

        self
    }

    pub fn open(path: &Path) -> Result<Self> {
        Self::open_with_cancellation(path, Arc::new(AtomicBool::new(false)))
    }

    pub fn open_with_cancellation(path: &Path, cancelled: Arc<AtomicBool>) -> Result<Self> {
        Self::open_with_initializer(path, cancelled, |_| Ok(()))
    }

    pub fn open_with_initializer(
        path: &Path,
        cancelled: Arc<AtomicBool>,
        initialize: fn(&Connection) -> Result<()>,
    ) -> Result<Self> {
        checkpoint("sqlite.before-open", &cancelled)?;
        let mut connection = Connection::open(path).map_err(storage)?;
        checkpoint("sqlite.after-open", &cancelled)?;
        connection.busy_timeout(LOCK_TIMEOUT).map_err(storage)?;

        // Check an existing database in one read snapshot before changing journal mode or schema.
        let read = connection
            .transaction_with_behavior(TransactionBehavior::Deferred)
            .map_err(storage)?;
        compatible_version(&read)?;
        read.commit().map_err(storage)?;

        connection
            .pragma_update(None, "foreign_keys", "ON")
            .map_err(storage)?;
        enable_wal(&connection, &cancelled)?;
        connection
            .pragma_update(None, "synchronous", "FULL")
            .map_err(storage)?;
        let tx = begin_transaction(&connection, &cancelled, LOCK_TIMEOUT)?;

        // Another opener may have initialized the database since the read-only check.
        match compatible_version(&tx)? {
            0 => {
                tx.execute_batch(include_str!("schema.sql"))
                    .map_err(storage)?;
                tx.pragma_update(None, "user_version", 4).map_err(storage)?;
            }
            2 => {
                tx.execute_batch(include_str!("migrate_v2.sql"))
                    .map_err(storage)?;
                tx.execute_batch(include_str!("migrate_v3.sql"))
                    .map_err(storage)?;
                tx.pragma_update(None, "user_version", 4).map_err(storage)?;
            }
            3 => {
                tx.execute_batch(include_str!("migrate_v3.sql"))
                    .map_err(storage)?;
                tx.pragma_update(None, "user_version", 4).map_err(storage)?;
            }
            _ => {}
        }

        let version: i64 = tx
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .map_err(storage)?;
        if version == 4 {
            tx.execute_batch(include_str!("migrate_v4.sql"))
                .map_err(storage)?;
            tx.pragma_update(None, "user_version", 5).map_err(storage)?;
        }

        initialize(&tx)?;

        checkpoint("sqlite.schema.before-commit", &cancelled)?;
        tx.commit().map_err(storage)?;
        checkpoint("sqlite.schema.after-commit", &cancelled)?;

        Ok(Self {
            connection: Rc::new(RefCell::new(connection)),
            cancelled,
            clock: Rc::new(SystemClock),
            directory: None,
        })
    }
}

impl Store for Sqlite {
    fn transaction<T>(
        &mut self,
        operation: impl FnOnce(&mut dyn UnitOfWork) -> Result<T>,
    ) -> Result<T> {
        checkpoint("sqlite.before-transaction", &self.cancelled)?;
        let connection = self.connection.borrow_mut();
        let tx = begin_transaction(&connection, &self.cancelled, LOCK_TIMEOUT)?;
        let result = operation(&mut SqliteUnitOfWork::new(
            &tx,
            &self.cancelled,
            self.clock.as_ref(),
            self.directory.as_ref(),
        ))?;
        checkpoint("sqlite.before-commit", &self.cancelled)?;
        tx.commit().map_err(storage)?;
        checkpoint("sqlite.after-commit", &self.cancelled)?;

        Ok(result)
    }
}
