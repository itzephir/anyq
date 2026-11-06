use crate::{sqlite_unit_of_work::SqliteUnitOfWork, storage_error::storage};
use anyq_queue_api::{Error, Result};
use anyq_queue_dispatch::ports::{Store, UnitOfWork};
use rusqlite::{Connection, TransactionBehavior};
use std::{cell::RefCell, path::Path, rc::Rc, time::Duration};

#[derive(Clone)]
pub struct Sqlite {
    pub(crate) connection: Rc<RefCell<Connection>>,
}

impl Sqlite {
    pub fn open(path: &Path) -> Result<Self> {
        let mut connection = Connection::open(path).map_err(storage)?;
        connection
            .busy_timeout(Duration::from_secs(5))
            .map_err(storage)?;
        connection
            .pragma_update(None, "foreign_keys", "ON")
            .map_err(storage)?;
        connection
            .pragma_update(None, "journal_mode", "WAL")
            .map_err(storage)?;
        connection
            .pragma_update(None, "synchronous", "FULL")
            .map_err(storage)?;
        let tx = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let version: i64 = tx
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .map_err(storage)?;

        if version != 0 && version != 1 {
            return Err(Error::Storage(format!(
                "unsupported database schema version {version}"
            )));
        }

        if version == 0 {
            tx.execute_batch(include_str!("schema.sql"))
                .map_err(storage)?;
            tx.pragma_update(None, "user_version", 1).map_err(storage)?;
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
