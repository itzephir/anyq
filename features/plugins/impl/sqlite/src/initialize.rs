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
        match version {
            1 => {
                crate::validate_existing::validate_version(connection, 1)?;
                connection.execute_batch("ALTER TABLE plugins ADD COLUMN catalog_revision INTEGER NOT NULL DEFAULT 0 CHECK(catalog_revision>=0); UPDATE plugin_schema SET version=2 WHERE id=1;").map_err(storage)?;
            }
            2 => crate::validate_existing::validate_existing(connection)?,
            _ => return Err(Error::Storage("unsupported plugin storage schema".into())),
        }
    }

    connection
        .execute_batch(include_str!("schema.sql"))
        .map_err(storage)
}
