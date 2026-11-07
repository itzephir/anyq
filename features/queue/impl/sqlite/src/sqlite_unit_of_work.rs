use crate::{sqlite_tasks::SqliteTasks, sqlite_topics::SqliteTopics};
use anyq_queue_dispatch::ports::{TaskRecords, TopicRecords, UnitOfWork};
use rusqlite::Connection;

pub(crate) struct SqliteUnitOfWork<'a> {
    topics: SqliteTopics<'a>,
    tasks: SqliteTasks<'a>,
}

impl<'a> SqliteUnitOfWork<'a> {
    pub fn new(connection: &'a Connection) -> Self {
        Self {
            topics: SqliteTopics { connection },
            tasks: SqliteTasks { connection },
        }
    }
}

impl UnitOfWork for SqliteUnitOfWork<'_> {
    fn topics(&mut self) -> &mut dyn TopicRecords {
        &mut self.topics
    }

    fn tasks(&mut self) -> &mut dyn TaskRecords {
        &mut self.tasks
    }
}
