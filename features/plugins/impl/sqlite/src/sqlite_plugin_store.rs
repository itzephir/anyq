use crate::{sqlite_records::SqliteRecords, storage::storage};
use anyq_plugin_dispatch::{Records, Store};
use anyq_queue_api::Result;
use rusqlite::{Connection, TransactionBehavior};
use std::{cell::RefCell, path::Path, rc::Rc, time::Duration};

#[derive(Clone)]
pub struct SqlitePluginStore {
    connection: Rc<RefCell<Connection>>,
}

impl SqlitePluginStore {
    pub fn open(path: &Path) -> Result<Self> {
        let connection = Connection::open(path).map_err(storage)?;
        connection
            .busy_timeout(Duration::from_secs(5))
            .map_err(storage)?;
        connection
            .pragma_update(None, "foreign_keys", "ON")
            .map_err(storage)?;

        Ok(Self {
            connection: Rc::new(RefCell::new(connection)),
        })
    }
}

impl Store for SqlitePluginStore {
    fn transaction<T>(
        &mut self,
        operation: impl FnOnce(&mut dyn Records) -> Result<T>,
    ) -> Result<T> {
        let mut connection = self.connection.borrow_mut();
        let tx = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let result = operation(&mut SqliteRecords { connection: &tx })?;
        tx.commit().map_err(storage)?;

        Ok(result)
    }
}
