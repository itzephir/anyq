use crate::{sqlite_unit_of_work::SqliteUnitOfWork, storage_error::storage};
use anyq_queue_api::{Error, Result};
use anyq_queue_dispatch::ports::{Store, UnitOfWork};
use rusqlite::{Connection, ErrorCode, TransactionBehavior};
use std::{
    cell::RefCell,
    path::Path,
    rc::Rc,
    thread,
    time::{Duration, Instant},
};

const LOCK_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone)]
pub struct Sqlite {
    pub(crate) connection: Rc<RefCell<Connection>>,
}

fn compatible_version(connection: &Connection) -> Result<i64> {
    let version: i64 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(storage)?;

    if version == 1 {
        return Err(Error::Storage(
            "schema version 1 contains legacy messages; choose a separate database with --db; existing data was not migrated".into()
        ));
    }

    if version != 0 && version != 2 {
        return Err(Error::Storage(format!(
            "unsupported database schema version {version}"
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

fn enable_wal(connection: &Connection) -> Result<()> {
    let deadline = Instant::now() + LOCK_TIMEOUT;

    loop {
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
    pub fn open(path: &Path) -> Result<Self> {
        let mut connection = Connection::open(path).map_err(storage)?;
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
        enable_wal(&connection)?;
        connection
            .pragma_update(None, "synchronous", "FULL")
            .map_err(storage)?;
        let tx = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;

        // Another opener may have initialized the database since the read-only check.
        if compatible_version(&tx)? == 0 {
            tx.execute_batch(include_str!("schema.sql"))
                .map_err(storage)?;
            tx.pragma_update(None, "user_version", 2).map_err(storage)?;
        }

        tx.commit().map_err(storage)?;

        Ok(Self {
            connection: Rc::new(RefCell::new(connection)),
        })
    }
}

impl Store for Sqlite {
    fn transaction<T>(
        &mut self,
        operation: impl FnOnce(&mut dyn UnitOfWork) -> Result<T>,
    ) -> Result<T> {
        let mut connection = self.connection.borrow_mut();
        let tx = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let result = operation(&mut SqliteUnitOfWork::new(&tx))?;
        tx.commit().map_err(storage)?;

        Ok(result)
    }
}
