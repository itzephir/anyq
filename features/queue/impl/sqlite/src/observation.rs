use crate::{
    Sqlite, checkpoint::checkpoint, sqlite_tasks::SqliteTasks, sqlite_topics::SqliteTopics,
    storage_error::storage,
};
use anyq_queue_api::{ObserveQueueRequest, QueueSnapshot, Result, TaskState};
use anyq_queue_dispatch::ports::{ObservationStore, TaskFilter, TaskRecords, TopicRecords};
use rusqlite::{TransactionBehavior, params};

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

        let topics = SqliteTopics {
            connection: &tx,
            cancelled: &self.cancelled,
        }
        .topics()?;
        let mut records = SqliteTasks {
            connection: &tx,
            cancelled: &self.cancelled,
        };
        let mut tasks = Vec::new();

        for state in states {
            checkpoint("sqlite.observation.read-state", &self.cancelled)?;
            tasks.extend(records.tasks(&TaskFilter {
                topic: None,
                state: *state,
                before_sequence: None,
            })?);
        }

        let snapshot = QueueSnapshot::new(topics, tasks)?;
        checkpoint("sqlite.observation.after-read", &self.cancelled)?;
        tx.commit().map_err(storage)?;

        Ok(snapshot)
    }
}
