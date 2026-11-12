use crate::storage_error::storage;
use anyq_queue_api::{Error, Result};
use rusqlite::Connection;

pub(crate) fn validate_runtime(connection: &Connection) -> Result<()> {
    for (table, query) in [
        (
            "topics",
            "SELECT name,capacity,autostart,metadata FROM topics LIMIT 0",
        ),
        (
            "tasks",
            "SELECT id,name,state,autostart_default,stopped,metadata FROM tasks LIMIT 0",
        ),
        (
            "task_topics",
            "SELECT task_id,topic_name FROM task_topics LIMIT 0",
        ),
        (
            "queue_identity",
            "SELECT id,identity FROM queue_identity LIMIT 0",
        ),
        (
            "queue_events",
            "SELECT sequence,payload FROM queue_events LIMIT 0",
        ),
        (
            "task_directories",
            "SELECT task_name,directory FROM task_directories LIMIT 0",
        ),
        (
            "managed_work",
            "SELECT owner,action_id,task_name,state,cancellation_requested,details FROM managed_work LIMIT 0",
        ),
        (
            "completion_intents",
            "SELECT task_name FROM completion_intents LIMIT 0",
        ),
        (
            "completion_receipts",
            "SELECT owner,action_id,task_name,force,result FROM completion_receipts LIMIT 0",
        ),
        (
            "sqlite_sequence",
            "SELECT name,seq FROM sqlite_sequence LIMIT 0",
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

    let identity: String = connection
        .query_row(
            "SELECT identity FROM queue_identity WHERE id=1",
            [],
            |row| row.get(0),
        )
        .map_err(storage)?;
    if identity.is_empty() {
        return Err(Error::Storage("missing queue event identity".into()));
    }

    Ok(())
}
