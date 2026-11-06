use crate::storage_error::storage;
use anyq_queue_api::{Message, Publish, Result};
use anyq_queue_dispatch::ports::MessageRecords;
use rusqlite::{Connection, OptionalExtension, Row, params};

pub(crate) struct SqliteMessages<'a> {
    pub connection: &'a Connection,
}

const MESSAGE_COLUMNS: &str = "id, topic, payload, dedup_key, max_attempts, created_at";

fn message(row: &Row<'_>) -> rusqlite::Result<Message> {
    Ok(Message {
        id: row.get(0)?,
        topic: row.get(1)?,
        payload: row.get(2)?,
        key: row.get(3)?,
        max_attempts: row.get(4)?,
        created_at: row.get(5)?,
    })
}

impl MessageRecords for SqliteMessages<'_> {
    fn message(&mut self, id: i64) -> Result<Option<Message>> {
        self.connection
            .query_row(
                &format!("SELECT {MESSAGE_COLUMNS} FROM messages WHERE id=?1"),
                [id],
                message,
            )
            .optional()
            .map_err(storage)
    }

    fn message_by_key(&mut self, topic: &str, key: &str) -> Result<Option<Message>> {
        self.connection
            .query_row(
                &format!("SELECT {MESSAGE_COLUMNS} FROM messages WHERE topic=?1 AND dedup_key=?2"),
                params![topic, key],
                message,
            )
            .optional()
            .map_err(storage)
    }

    fn insert_message(&mut self, p: &Publish, now: i64) -> Result<Message> {
        self.connection.execute("INSERT INTO messages(topic,payload,dedup_key,max_attempts,created_at) VALUES (?1,?2,?3,?4,?5)", params![p.topic,p.payload,p.key,p.max_attempts,now]).map_err(storage)?;

        Ok(Message {
            id: self.connection.last_insert_rowid(),
            topic: p.topic.clone(),
            payload: p.payload.clone(),
            key: p.key.clone(),
            max_attempts: p.max_attempts,
            created_at: now,
        })
    }

    fn messages(&mut self, topic: Option<&str>, after: i64, limit: u32) -> Result<Vec<Message>> {
        let mut s = self.connection.prepare(&format!("SELECT {MESSAGE_COLUMNS} FROM messages WHERE (?1 IS NULL OR topic=?1) AND id>?2 ORDER BY id LIMIT ?3")).map_err(storage)?;
        s.query_map(params![topic, after, limit], message)
            .map_err(storage)?
            .collect::<rusqlite::Result<_>>()
            .map_err(storage)
    }
}
