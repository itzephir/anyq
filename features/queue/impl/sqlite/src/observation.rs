use crate::{Sqlite, checkpoint::checkpoint, storage_error::storage};
use anyq_queue_api::{
    ObserveQueueRequest, ObservedTask, ObservedTopic, QueueSnapshot, Result, TaskState,
};
use anyq_queue_dispatch::ports::ObservationStore;
use rusqlite::{Connection, TransactionBehavior, params};
use std::sync::atomic::AtomicBool;

fn topics(connection: &Connection, cancelled: &AtomicBool) -> Result<Vec<ObservedTopic>> {
    let mut query = connection
        .prepare("SELECT name,capacity,autostart FROM topics ORDER BY name")
        .map_err(storage)?;
    let rows = query
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, u32>(1)?,
                row.get::<_, Option<bool>>(2)?,
            ))
        })
        .map_err(storage)?;
    let mut topics = Vec::new();

    for row in rows {
        checkpoint("sqlite.observation.read-topic", cancelled)?;
        let (name, capacity, autostart) = row.map_err(storage)?;
        topics.push(ObservedTopic::restore(name, capacity, autostart)?);
    }

    Ok(topics)
}

fn tasks(
    connection: &Connection,
    cancelled: &AtomicBool,
    state: TaskState,
) -> Result<Vec<ObservedTask>> {
    // Select only the shared queue fields. Large metadata/history is never loaded.
    let mut query = connection
        .prepare("SELECT id,name,autostart_default,stopped FROM tasks WHERE state=?1 ORDER BY id")
        .map_err(storage)?;
    let rows = query
        .query_map([state.as_str()], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, bool>(2)?,
                row.get::<_, bool>(3)?,
            ))
        })
        .map_err(storage)?;
    let mut tasks = Vec::new();

    for row in rows {
        checkpoint("sqlite.observation.read-task", cancelled)?;
        let (sequence, name, autostart_default, stopped) = row.map_err(storage)?;
        let topics = connection
            .prepare("SELECT topic_name FROM task_topics WHERE task_id=?1 ORDER BY topic_name")
            .map_err(storage)?
            .query_map([sequence], |row| row.get::<_, String>(0))
            .map_err(storage)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(storage)?;
        tasks.push(ObservedTask::restore(
            sequence,
            name,
            state,
            topics,
            autostart_default,
            stopped,
        )?);
    }

    Ok(tasks)
}

impl ObservationStore for Sqlite {
    fn snapshot(
        &mut self,
        states: &[TaskState; 2],
        request: ObserveQueueRequest,
    ) -> Result<QueueSnapshot> {
        checkpoint("sqlite.observation.before-read", &self.cancelled)?;
        let mut connection = self.connection.borrow_mut();
        let tx = connection
            .transaction_with_behavior(TransactionBehavior::Deferred)
            .map_err(storage)?;
        let (entities, text_bytes): (i64, i64) = tx.query_row(
            "WITH current AS (SELECT id,name FROM tasks WHERE state IN (?1,?2)),
             links AS (SELECT topic_name FROM task_topics WHERE task_id IN (SELECT id FROM current))
             SELECT (SELECT count(*) FROM topics) + (SELECT count(*) FROM current) + (SELECT count(*) FROM links),
             COALESCE((SELECT sum(length(CAST(name AS BLOB))) FROM topics),0)
             + COALESCE((SELECT sum(length(CAST(name AS BLOB))) FROM current),0)
             + COALESCE((SELECT sum(length(CAST(topic_name AS BLOB))) FROM links),0)",
            params![states[0].as_str(), states[1].as_str()], |row| Ok((row.get(0)?, row.get(1)?)),
        ).map_err(storage)?;
        request.require_size(entities as u64, text_bytes as u64)?;

        let topics = topics(&tx, &self.cancelled)?;
        let mut observed_tasks = Vec::new();
        for state in states {
            observed_tasks.extend(tasks(&tx, &self.cancelled, *state)?);
        }

        let snapshot = QueueSnapshot::new(topics, observed_tasks)?;
        checkpoint("sqlite.observation.after-read", &self.cancelled)?;
        tx.commit().map_err(storage)?;

        Ok(snapshot)
    }
}
