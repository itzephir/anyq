use crate::{Sqlite, storage_error::storage};
use anyq_queue_api::{Error, QueueEvent, ReadQueueEvents, Result};
use anyq_queue_wire::EventDto;
use rusqlite::params;
use serde::Deserialize;

impl ReadQueueEvents for Sqlite {
    fn after(&mut self, sequence: i64, limit: usize) -> Result<Vec<QueueEvent>> {
        let connection = self.connection.borrow();
        let mut statement = connection.prepare(
            "SELECT sequence,payload FROM queue_events WHERE sequence>?1 ORDER BY sequence LIMIT ?2",
        ).map_err(storage)?;
        let rows = statement
            .query_map(params![sequence, limit.min(256) as i64], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(storage)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(storage)?;

        rows.into_iter()
            .map(|(sequence, payload)| {
                let mut decoder = serde_json::Deserializer::from_str(&payload);
                decoder.disable_recursion_limit();
                let event = EventDto::deserialize(&mut decoder)
                    .map_err(|error| Error::Storage(error.to_string()))?;
                decoder
                    .end()
                    .map_err(|error| Error::Storage(error.to_string()))?;
                if event.sequence != sequence {
                    return Err(Error::Storage(
                        "queue event identity disagrees with its record".into(),
                    ));
                }

                event.try_into()
            })
            .collect()
    }

    fn watermark(&mut self) -> Result<i64> {
        self.connection
            .borrow()
            .query_row(
                "SELECT coalesce((SELECT seq FROM sqlite_sequence WHERE name='queue_events'),0)",
                [],
                |row| row.get(0),
            )
            .map_err(storage)
    }
}
