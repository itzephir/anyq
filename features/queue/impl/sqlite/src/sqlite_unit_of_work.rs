use crate::{
    sqlite_deliveries::SqliteDeliveries, sqlite_groups::SqliteGroups,
    sqlite_messages::SqliteMessages, sqlite_topics::SqliteTopics,
};

use anyq_queue_dispatch::ports::*;
use rusqlite::Connection;

pub(crate) struct SqliteUnitOfWork<'a> {
    topics: SqliteTopics<'a>,
    groups: SqliteGroups<'a>,
    messages: SqliteMessages<'a>,
    deliveries: SqliteDeliveries<'a>,
}

impl<'a> SqliteUnitOfWork<'a> {
    pub fn new(connection: &'a Connection) -> Self {
        Self {
            topics: SqliteTopics { connection },
            groups: SqliteGroups { connection },
            messages: SqliteMessages { connection },
            deliveries: SqliteDeliveries { connection },
        }
    }
}

impl UnitOfWork for SqliteUnitOfWork<'_> {
    fn topics(&mut self) -> &mut dyn TopicRecords {
        &mut self.topics
    }

    fn groups(&mut self) -> &mut dyn GroupRecords {
        &mut self.groups
    }

    fn messages(&mut self) -> &mut dyn MessageRecords {
        &mut self.messages
    }

    fn deliveries(&mut self) -> &mut dyn DeliveryRecords {
        &mut self.deliveries
    }
}
