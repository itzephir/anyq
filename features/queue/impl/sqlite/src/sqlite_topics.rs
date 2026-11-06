use crate::storage_error::storage;
use anyq_queue_api::{Result, Topic};
use anyq_queue_dispatch::ports::TopicRecords;
use rusqlite::{Connection, OptionalExtension, params};

pub(crate) struct SqliteTopics<'a> {
    pub connection: &'a Connection,
}
impl TopicRecords for SqliteTopics<'_> {
    fn topics(&mut self) -> Result<Vec<Topic>> {
        let mut s = self
            .connection
            .prepare("SELECT name, capacity FROM topics ORDER BY name")
            .map_err(storage)?;
        s.query_map([], |r| {
            Ok(Topic {
                name: r.get(0)?,
                capacity: r.get(1)?,
            })
        })
        .map_err(storage)?
        .collect::<rusqlite::Result<_>>()
        .map_err(storage)
    }
    fn topic(&mut self, name: &str) -> Result<Option<Topic>> {
        self.connection
            .query_row(
                "SELECT name, capacity FROM topics WHERE name=?1",
                [name],
                |r| {
                    Ok(Topic {
                        name: r.get(0)?,
                        capacity: r.get(1)?,
                    })
                },
            )
            .optional()
            .map_err(storage)
    }
    fn put_topic(&mut self, t: &Topic) -> Result<()> {
        self.connection.execute("INSERT INTO topics(name, capacity) VALUES (?1,?2) ON CONFLICT(name) DO UPDATE SET capacity=excluded.capacity", params![t.name,t.capacity]).map_err(storage)?;
        Ok(())
    }
}
