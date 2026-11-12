use crate::storage::storage;
use anyq_queue_api::{Error, Result};
use rusqlite::Connection;

/// The daemon injects this initializer into the queue's open transaction.
pub fn initialize(connection: &Connection) -> Result<()> {
    let exists: bool = connection.query_row("SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE type='table' AND name='plugin_schema')", [], |row| row.get(0)).map_err(storage)?;
    if exists {
        let version: i64 = connection
            .query_row("SELECT version FROM plugin_schema WHERE id=1", [], |row| {
                row.get(0)
            })
            .map_err(storage)?;
        if version != 1 {
            return Err(Error::Storage("unsupported plugin storage schema".into()));
        }
    }

    connection
        .execute_batch(include_str!("schema.sql"))
        .map_err(storage)
}
