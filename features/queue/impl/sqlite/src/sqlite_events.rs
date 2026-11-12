use crate::storage_error::storage;
use anyq_queue_api::{Error, EventSnapshot, ExecutionDirectory, QueueEvent, Result};
use anyq_queue_dispatch::ports::EventRecords;
use anyq_queue_wire::{CallerDirectoryDto, EventDto};
use rusqlite::{Connection, OptionalExtension, params};

pub(crate) struct SqliteEvents<'a> {
    pub connection: &'a Connection,
}

impl EventRecords for SqliteEvents<'_> {
    fn append(&mut self, snapshot: &EventSnapshot) -> Result<()> {
        let database_id = self
            .connection
            .query_row(
                "SELECT identity FROM queue_identity WHERE id=1",
                [],
                |row| row.get(0),
            )
            .map_err(storage)?;
        self.connection
            .execute("INSERT INTO queue_events(payload) VALUES ('{}')", [])
            .map_err(storage)?;
        let sequence = self.connection.last_insert_rowid();
        let dto = EventDto::try_from(QueueEvent {
            database_id,
            sequence,
            version: 1,
            snapshot: snapshot.clone(),
        })?;
        let payload =
            serde_json::to_string(&dto).map_err(|error| Error::Storage(error.to_string()))?;

        self.connection
            .execute(
                "UPDATE queue_events SET payload=?1 WHERE sequence=?2",
                params![payload, sequence],
            )
            .map_err(storage)?;

        Ok(())
    }

    fn save_directory(&mut self, task: &str, directory: &ExecutionDirectory) -> Result<()> {
        let directory = serde_json::to_string(&CallerDirectoryDto::try_from(directory.clone())?)
            .map_err(|error| Error::Storage(error.to_string()))?;
        self.connection
            .execute(
                "INSERT INTO task_directories(task_name,directory) VALUES (?1,?2)",
                params![task, directory],
            )
            .map_err(storage)?;

        Ok(())
    }

    fn directory(&mut self, task: &str) -> Result<Option<ExecutionDirectory>> {
        let value: Option<String> = self
            .connection
            .query_row(
                "SELECT directory FROM task_directories WHERE task_name=?1",
                [task],
                |row| row.get(0),
            )
            .optional()
            .map_err(storage)?;

        value
            .map(|value| {
                serde_json::from_str::<CallerDirectoryDto>(&value)
                    .map(Into::into)
                    .map_err(|error| Error::Storage(error.to_string()))
            })
            .transpose()
    }
}
