use crate::sqlite_work::SqliteWork;
use crate::{sqlite_events::SqliteEvents, sqlite_tasks::SqliteTasks, sqlite_topics::SqliteTopics};
use anyq_queue_api::ExecutionDirectory;
use anyq_queue_dispatch::ports::{Clock, EventRecords, WorkRecords};
use anyq_queue_dispatch::ports::{TaskRecords, TopicRecords, UnitOfWork};
use rusqlite::Connection;
use std::sync::atomic::AtomicBool;

pub(crate) struct SqliteUnitOfWork<'a> {
    topics: SqliteTopics<'a>,
    events: SqliteEvents<'a>,
    work: SqliteWork<'a>,
    clock: &'a dyn Clock,
    directory: Option<&'a ExecutionDirectory>,
    tasks: SqliteTasks<'a>,
}

impl<'a> SqliteUnitOfWork<'a> {
    pub fn new(
        connection: &'a Connection,
        cancelled: &'a AtomicBool,
        clock: &'a dyn Clock,
        directory: Option<&'a ExecutionDirectory>,
    ) -> Self {
        Self {
            events: SqliteEvents { connection },
            work: SqliteWork { connection },
            clock,
            directory,
            topics: SqliteTopics {
                connection,
                cancelled,
            },
            tasks: SqliteTasks {
                connection,
                cancelled,
            },
        }
    }
}

impl UnitOfWork for SqliteUnitOfWork<'_> {
    fn work(&mut self) -> &mut dyn WorkRecords {
        &mut self.work
    }

    fn clock(&self) -> &dyn Clock {
        self.clock
    }

    fn caller_directory(&self) -> Option<&ExecutionDirectory> {
        self.directory
    }

    fn events(&mut self) -> &mut dyn EventRecords {
        &mut self.events
    }

    fn topics(&mut self) -> &mut dyn TopicRecords {
        &mut self.topics
    }

    fn tasks(&mut self) -> &mut dyn TaskRecords {
        &mut self.tasks
    }
}
