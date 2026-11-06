//! SQLite implements storage primitives. Delivery policy belongs to dispatch/API.
#[cfg(test)]
mod tests;
use anyq_queue_api::*;
use anyq_queue_dispatch::ports::{DeliveryFilter, Records, Store};
use rusqlite::{Connection, OptionalExtension, Row, TransactionBehavior, params};
use std::{path::Path, time::Duration};

fn storage(error: rusqlite::Error) -> Error {
    Error::Storage(error.to_string())
}
pub struct Sqlite {
    connection: Connection,
}
impl Sqlite {
    pub fn open(path: &Path) -> Result<Self> {
        let mut connection = Connection::open(path).map_err(storage)?;
        connection
            .busy_timeout(Duration::from_secs(5))
            .map_err(storage)?;
        connection
            .pragma_update(None, "foreign_keys", "ON")
            .map_err(storage)?;
        connection
            .pragma_update(None, "journal_mode", "WAL")
            .map_err(storage)?;
        connection
            .pragma_update(None, "synchronous", "FULL")
            .map_err(storage)?;
        let tx = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let version: i64 = tx
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .map_err(storage)?;
        if version != 0 && version != 1 {
            return Err(Error::Storage(format!(
                "unsupported database schema version {version}"
            )));
        }
        if version == 0 {
            tx.execute_batch(include_str!("schema.sql"))
                .map_err(storage)?;
            tx.pragma_update(None, "user_version", 1).map_err(storage)?;
        }
        tx.commit().map_err(storage)?;
        Ok(Self { connection })
    }
}
impl Store for Sqlite {
    fn transaction<T>(
        &mut self,
        operation: impl FnOnce(&mut dyn Records) -> Result<T>,
    ) -> Result<T> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let result = operation(&mut SqlRecords { connection: &tx })?;
        tx.commit().map_err(storage)?;
        Ok(result)
    }
}
struct SqlRecords<'a> {
    connection: &'a Connection,
}
const MESSAGE_COLUMNS: &str = "id, topic, payload, dedup_key, max_attempts, created_at";
fn message(row: &Row<'_>) -> rusqlite::Result<Message> {
    Ok(Message {
        id: row.get(0)?,
        topic: row.get(1)?,
        payload: row.get(2)?,
        key: row.get(3)?,
        max_attempts: row.get(4)?,
        created_at: row.get(5)?,
    })
}
fn delivery(row: &Row<'_>) -> rusqlite::Result<Delivery> {
    let value: String = row.get(2)?;
    let state = match value.as_str() {
        "pending" => State::Pending,
        "running" => State::Running,
        "succeeded" => State::Succeeded,
        "failed" => State::Failed,
        _ => {
            return Err(rusqlite::Error::InvalidColumnType(
                2,
                "state".into(),
                rusqlite::types::Type::Text,
            ));
        }
    };
    Ok(Delivery {
        message_id: row.get(0)?,
        group: row.get(1)?,
        state,
        attempts: row.get(3)?,
        max_attempts: row.get(4)?,
        available_at: row.get(5)?,
        consumer: row.get(6)?,
        token: row.get(7)?,
        lease_until: row.get(8)?,
        finished_at: row.get(9)?,
        result: row.get(10)?,
        last_error: row.get(11)?,
    })
}
// Exact optional filters are defined by the use case, not selected by the DB adapter.
const DELIVERY_WHERE: &str = "FROM deliveries d JOIN messages m ON m.id = d.message_id
    WHERE (?1 IS NULL OR m.topic = ?1) AND (?2 IS NULL OR d.group_name = ?2)
    AND (?3 IS NULL OR d.message_id = ?3) AND (?4 IS NULL OR d.state = ?4)
    AND (?5 IS NULL OR d.available_at <= ?5) AND (?6 IS NULL OR d.lease_until <= ?6)";
impl Records for SqlRecords<'_> {
    fn topics(&mut self) -> Result<Vec<Topic>> {
        let mut s = self
            .connection
            .prepare("SELECT name, capacity FROM topics ORDER BY name")
            .map_err(storage)?;
        s.query_map([], |r| {
            Ok(Topic {
                name: r.get(0)?,
                capacity: r.get(1)?,
            })
        })
        .map_err(storage)?
        .collect::<rusqlite::Result<_>>()
        .map_err(storage)
    }
    fn topic(&mut self, name: &str) -> Result<Option<Topic>> {
        self.connection
            .query_row(
                "SELECT name, capacity FROM topics WHERE name=?1",
                [name],
                |r| {
                    Ok(Topic {
                        name: r.get(0)?,
                        capacity: r.get(1)?,
                    })
                },
            )
            .optional()
            .map_err(storage)
    }
    fn put_topic(&mut self, t: &Topic) -> Result<()> {
        self.connection.execute("INSERT INTO topics(name, capacity) VALUES (?1,?2) ON CONFLICT(name) DO UPDATE SET capacity=excluded.capacity", params![t.name,t.capacity]).map_err(storage)?;
        Ok(())
    }
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
    fn seed_group(&mut self, g: &ConsumerGroup) -> Result<()> {
        self.connection.execute("INSERT INTO deliveries(message_id,group_name,state,attempts,available_at) SELECT id,?2,'pending',0,created_at FROM messages WHERE topic=?1", params![g.topic,g.name]).map_err(storage)?;
        Ok(())
    }
    fn message(&mut self, id: i64) -> Result<Option<Message>> {
        self.connection
            .query_row(
                &format!("SELECT {MESSAGE_COLUMNS} FROM messages WHERE id=?1"),
                [id],
                message,
            )
            .optional()
            .map_err(storage)
    }
    fn message_by_key(&mut self, topic: &str, key: &str) -> Result<Option<Message>> {
        self.connection
            .query_row(
                &format!("SELECT {MESSAGE_COLUMNS} FROM messages WHERE topic=?1 AND dedup_key=?2"),
                params![topic, key],
                message,
            )
            .optional()
            .map_err(storage)
    }
    fn insert_message(&mut self, p: &Publish, now: i64) -> Result<Message> {
        self.connection.execute("INSERT INTO messages(topic,payload,dedup_key,max_attempts,created_at) VALUES (?1,?2,?3,?4,?5)", params![p.topic,p.payload,p.key,p.max_attempts,now]).map_err(storage)?;
        Ok(Message {
            id: self.connection.last_insert_rowid(),
            topic: p.topic.clone(),
            payload: p.payload.clone(),
            key: p.key.clone(),
            max_attempts: p.max_attempts,
            created_at: now,
        })
    }
    fn messages(&mut self, topic: Option<&str>, after: i64, limit: u32) -> Result<Vec<Message>> {
        let mut s = self.connection.prepare(&format!("SELECT {MESSAGE_COLUMNS} FROM messages WHERE (?1 IS NULL OR topic=?1) AND id>?2 ORDER BY id LIMIT ?3")).map_err(storage)?;
        s.query_map(params![topic, after, limit], message)
            .map_err(storage)?
            .collect::<rusqlite::Result<_>>()
            .map_err(storage)
    }
    fn deliveries(&mut self, f: &DeliveryFilter) -> Result<Vec<Delivery>> {
        let mut s = self.connection.prepare(&format!("SELECT d.message_id,d.group_name,d.state,d.attempts,m.max_attempts,d.available_at,d.consumer,d.token,d.lease_until,d.finished_at,d.result,d.last_error {DELIVERY_WHERE} ORDER BY d.message_id,d.group_name LIMIT ?7")).map_err(storage)?;
        s.query_map(
            params![
                f.topic,
                f.group,
                f.message_id,
                f.state.map(State::as_str),
                f.ready_at,
                f.expired_at,
                f.limit.map(i64::from).unwrap_or(-1)
            ],
            delivery,
        )
        .map_err(storage)?
        .collect::<rusqlite::Result<_>>()
        .map_err(storage)
    }
    fn count_deliveries(&mut self, f: &DeliveryFilter) -> Result<u64> {
        self.connection
            .query_row(
                &format!("SELECT count(*) {DELIVERY_WHERE}"),
                params![
                    f.topic,
                    f.group,
                    f.message_id,
                    f.state.map(State::as_str),
                    f.ready_at,
                    f.expired_at
                ],
                |r| r.get::<_, i64>(0),
            )
            .map(|n| n as u64)
            .map_err(storage)
    }
    fn put_delivery(&mut self, d: &Delivery) -> Result<()> {
        self.connection.execute("INSERT INTO deliveries(message_id,group_name,state,attempts,available_at,consumer,token,lease_until,finished_at,result,last_error) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11) ON CONFLICT(message_id,group_name) DO UPDATE SET state=excluded.state,attempts=excluded.attempts,available_at=excluded.available_at,consumer=excluded.consumer,token=excluded.token,lease_until=excluded.lease_until,finished_at=excluded.finished_at,result=excluded.result,last_error=excluded.last_error", params![d.message_id,d.group,d.state.as_str(),d.attempts,d.available_at,d.consumer,d.token,d.lease_until,d.finished_at,d.result,d.last_error]).map_err(storage)?;
        Ok(())
    }
}
