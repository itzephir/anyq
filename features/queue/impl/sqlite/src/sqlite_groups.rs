use crate::storage_error::storage;
use anyq_queue_api::{ConsumerGroup, Result};
use anyq_queue_dispatch::ports::GroupRecords;
use rusqlite::{Connection, params};

pub(crate) struct SqliteGroups<'a> {
    pub connection: &'a Connection,
}
impl GroupRecords for SqliteGroups<'_> {
    fn groups(&mut self, topic: &str) -> Result<Vec<ConsumerGroup>> {
        let mut s = self
            .connection
            .prepare("SELECT topic,name FROM consumer_groups WHERE topic=?1 ORDER BY name")
            .map_err(storage)?;
        s.query_map([topic], |r| {
            Ok(ConsumerGroup {
                topic: r.get(0)?,
                name: r.get(1)?,
            })
        })
        .map_err(storage)?
        .collect::<rusqlite::Result<_>>()
        .map_err(storage)
    }
    fn insert_group(&mut self, g: &ConsumerGroup) -> Result<()> {
        self.connection
            .execute(
                "INSERT INTO consumer_groups(topic,name) VALUES (?1,?2)",
                params![g.topic, g.name],
            )
            .map_err(storage)?;
        Ok(())
    }
}
