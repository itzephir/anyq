use crate::checkpoint::checkpoint;
use crate::storage_error::storage;
use anyq_queue_api::{Error, Result, Topic};
use anyq_queue_dispatch::ports::TopicRecords;
use rusqlite::{Connection, OptionalExtension, params};
use std::sync::atomic::AtomicBool;

pub(crate) struct SqliteTopics<'a> {
    pub connection: &'a Connection,
    pub cancelled: &'a AtomicBool,
}

fn restore_topic((name, capacity): (String, u32)) -> Result<Topic> {
    Topic::restore(name, capacity).map_err(|error| Error::Storage(error.to_string()))
}

impl TopicRecords for SqliteTopics<'_> {
    fn topics(&mut self) -> Result<Vec<Topic>> {
        let mut s = self
            .connection
            .prepare("SELECT name, capacity FROM topics ORDER BY name")
            .map_err(storage)?;
        s.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(storage)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(storage)?
            .into_iter()
            .map(restore_topic)
            .collect()
    }

    fn topic(&mut self, name: &str) -> Result<Option<Topic>> {
        self.connection
            .query_row(
                "SELECT name, capacity FROM topics WHERE name=?1",
                [name],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(storage)?
            .map(restore_topic)
            .transpose()
    }

    fn put_topic(&mut self, t: &Topic) -> Result<()> {
        let topic = Topic::restore(t.name.clone(), t.capacity)?;

        self.connection
            .execute(
                "INSERT INTO topics(name, capacity) VALUES (?1,?2)
                 ON CONFLICT(name) DO UPDATE SET capacity=excluded.capacity",
                params![topic.name, topic.capacity],
            )
            .map_err(storage)?;

        checkpoint("sqlite.after-topic-write", self.cancelled)?;

        Ok(())
    }
}
