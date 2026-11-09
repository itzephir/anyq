use crate::checkpoint::checkpoint;
use crate::storage_error::storage;
use anyq_queue_api::{Error, Result, Task, TaskState};
use anyq_queue_dispatch::ports::{TaskFilter, TaskRecords};
use rusqlite::{Connection, OptionalExtension, params};
use std::sync::atomic::AtomicBool;

const MATCHING_TASKS: &str = "FROM tasks t
    WHERE (?1 IS NULL OR EXISTS (SELECT 1 FROM task_topics m
        WHERE m.task_id=t.id AND m.topic_name=?1))
    AND t.state=?2 AND (?3 IS NULL OR t.id<?3)";

pub(crate) struct SqliteTasks<'a> {
    pub connection: &'a Connection,
    pub cancelled: &'a AtomicBool,
}

fn state(value: &str) -> Result<TaskState> {
    match value {
        "registered" => Ok(TaskState::Registered),
        "queued" => Ok(TaskState::Queued),
        "running" => Ok(TaskState::Running),
        "completed" => Ok(TaskState::Completed),
        _ => Err(Error::Storage(format!("unknown task state '{value}'"))),
    }
}

impl TaskRecords for SqliteTasks<'_> {
    fn tasks(&mut self, filter: &TaskFilter) -> Result<Vec<Task>> {
        let query = format!("SELECT t.name {MATCHING_TASKS} ORDER BY t.id");
        let names = self
            .connection
            .prepare(&query)
            .map_err(storage)?
            .query_map(
                params![filter.topic, filter.state.as_str(), filter.before_sequence],
                |row| row.get::<_, String>(0),
            )
            .map_err(storage)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(storage)?;

        names
            .into_iter()
            .map(|name| {
                self.task(&name)?.ok_or_else(|| {
                    Error::Storage(format!("task '{name}' disappeared inside a transaction"))
                })
            })
            .collect()
    }

    fn task_names(&mut self) -> Result<Vec<String>> {
        self.connection
            .prepare("SELECT name FROM tasks ORDER BY name")
            .map_err(storage)?
            .query_map([], |row| row.get(0))
            .map_err(storage)?
            .collect::<rusqlite::Result<_>>()
            .map_err(storage)
    }

    fn task(&mut self, name: &str) -> Result<Option<Task>> {
        let row: Option<(i64, String, String, bool, bool)> = self
            .connection
            .query_row(
                "SELECT id,name,state,autostart_default,stopped FROM tasks WHERE name=?1",
                [name],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                    ))
                },
            )
            .optional()
            .map_err(storage)?;

        let Some((sequence, name, raw_state, autostart_default, stopped)) = row else {
            return Ok(None);
        };

        let mut statement = self
            .connection
            .prepare("SELECT topic_name FROM task_topics WHERE task_id=?1 ORDER BY topic_name")
            .map_err(storage)?;
        let topics = statement
            .query_map([sequence], |row| row.get::<_, String>(0))
            .map_err(storage)?
            .collect::<rusqlite::Result<_>>()
            .map_err(storage)?;

        Task::restore(sequence, name, state(&raw_state)?, topics)
            .map(|task| Some(task.with_start_policy(autostart_default, stopped)))
            .map_err(|error| Error::Storage(error.to_string()))
    }

    fn insert_task(&mut self, name: &str) -> Result<Task> {
        self.connection
            .execute(
                "INSERT INTO tasks(name,state) VALUES (?1,'registered')",
                [name],
            )
            .map_err(storage)?;

        checkpoint("sqlite.after-task-insert", self.cancelled)?;

        Task::restore(
            self.connection.last_insert_rowid(),
            name.into(),
            TaskState::Registered,
            Vec::new(),
        )
    }

    fn save_task(&mut self, task: &Task) -> Result<()> {
        let changed = self
            .connection
            .execute(
                "UPDATE tasks SET state=?1,autostart_default=?4,stopped=?5 WHERE id=?2 AND name=?3",
                params![
                    task.state().as_str(),
                    task.sequence(),
                    task.name(),
                    task.autostart_default(),
                    task.stopped()
                ],
            )
            .map_err(storage)?;

        if changed != 1 {
            return Err(Error::Storage("cannot save a missing task".into()));
        }

        checkpoint("sqlite.after-task-update", self.cancelled)?;

        self.connection
            .execute(
                "DELETE FROM task_topics WHERE task_id=?1",
                [task.sequence()],
            )
            .map_err(storage)?;

        for topic in task.topics() {
            self.connection
                .execute(
                    "INSERT INTO task_topics(task_id,topic_name) VALUES (?1,?2)",
                    params![task.sequence(), topic],
                )
                .map_err(storage)?;
            checkpoint("sqlite.after-membership-write", self.cancelled)?;
        }

        Ok(())
    }

    fn count_tasks(&mut self, filter: &TaskFilter) -> Result<u64> {
        self.connection
            .query_row(
                &format!("SELECT count(*) {MATCHING_TASKS}"),
                params![filter.topic, filter.state.as_str(), filter.before_sequence],
                |row| row.get::<_, i64>(0),
            )
            .map(|count| count as u64)
            .map_err(storage)
    }
}
