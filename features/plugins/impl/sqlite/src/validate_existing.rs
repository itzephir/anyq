use crate::storage::storage;
use anyq_queue_api::{Error, Result};
use rusqlite::Connection;

pub fn validate_existing(connection: &Connection) -> Result<()> {
    for (table, query) in [
        (
            "plugin_schema",
            "SELECT id,version FROM plugin_schema LIMIT 0",
        ),
        (
            "plugins",
            "SELECT name,executable,arguments,enabled,running,cursor,failures,retry_at,last_failure,subscriptions,capabilities,state FROM plugins LIMIT 0",
        ),
        (
            "plugin_timers",
            "SELECT plugin,id,deadline,origin_sequence,payload FROM plugin_timers LIMIT 0",
        ),
    ] {
        let exists: bool = connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE type='table' AND name=?1)",
                [table],
                |row| row.get(0),
            )
            .map_err(storage)?;
        if !exists {
            return Err(Error::Storage(format!("missing runtime table '{table}'")));
        }
        connection.prepare(query).map_err(storage)?;
    }

    let version: i64 = connection
        .query_row("SELECT version FROM plugin_schema WHERE id=1", [], |row| {
            row.get(0)
        })
        .map_err(storage)?;
    if version != 1 {
        return Err(Error::Storage("unsupported plugin storage schema".into()));
    }

    Ok(())
}
