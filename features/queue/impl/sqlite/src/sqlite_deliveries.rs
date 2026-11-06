use crate::storage_error::storage;
use anyq_queue_api::{ConsumerGroup, Delivery, Result, State};
use anyq_queue_dispatch::ports::{DeliveryFilter, DeliveryRecords};
use rusqlite::{Connection, Row, params};

pub(crate) struct SqliteDeliveries<'a> {
    pub connection: &'a Connection,
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

impl DeliveryRecords for SqliteDeliveries<'_> {
    fn seed_group(&mut self, g: &ConsumerGroup) -> Result<()> {
        self.connection.execute("INSERT INTO deliveries(message_id,group_name,state,attempts,available_at) SELECT id,?2,'pending',0,created_at FROM messages WHERE topic=?1", params![g.topic,g.name]).map_err(storage)?;

        Ok(())
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
