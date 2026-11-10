use crate::storage::{conversion, storage};
use anyq_plugin_api::{PluginRegistration, PluginTimer};
use anyq_plugin_dispatch::Records;
use anyq_plugin_wire::{decode, encode};
use anyq_queue_api::{Error, Metadata, Result};
use anyq_queue_wire::MetadataDto;
use rusqlite::{Connection, OptionalExtension, Row, params};

pub(crate) struct SqliteRecords<'a> {
    pub connection: &'a Connection,
}

fn metadata(value: &str) -> Result<Metadata> {
    decode::<MetadataDto>(value.as_bytes())?.try_into()
}

fn read_registration(row: &Row<'_>) -> rusqlite::Result<PluginRegistration> {
    Ok(PluginRegistration {
        name: row.get("name")?,
        executable: row.get("executable")?,
        arguments: serde_json::from_str(&row.get::<_, String>("arguments")?).map_err(conversion)?,
        enabled: row.get("enabled")?,
        running: row.get("running")?,
        cursor: row.get("cursor")?,
        failures: row.get("failures")?,
        retry_at: row.get("retry_at")?,
        last_failure: row.get("last_failure")?,
        subscriptions: serde_json::from_str(&row.get::<_, String>("subscriptions")?)
            .map_err(conversion)?,
        capabilities: serde_json::from_str(&row.get::<_, String>("capabilities")?)
            .map_err(conversion)?,
        state: metadata(&row.get::<_, String>("state")?).map_err(conversion)?,
    })
}

fn json<T: serde::Serialize>(value: &T) -> Result<String> {
    serde_json::to_string(value).map_err(|error| Error::Storage(error.to_string()))
}

impl Records for SqliteRecords<'_> {
    fn prune_events_before(&mut self, sequence: i64) -> Result<()> {
        self.connection
            .execute("DELETE FROM queue_events WHERE sequence<?1", [sequence])
            .map_err(storage)?;

        Ok(())
    }

    fn registration(&mut self, name: &str) -> Result<Option<PluginRegistration>> {
        self.connection
            .query_row(
                "SELECT * FROM plugins WHERE name=?1",
                [name],
                read_registration,
            )
            .optional()
            .map_err(storage)
    }

    fn registrations(&mut self) -> Result<Vec<PluginRegistration>> {
        self.connection
            .prepare("SELECT * FROM plugins ORDER BY name")
            .map_err(storage)?
            .query_map([], read_registration)
            .map_err(storage)?
            .collect::<rusqlite::Result<_>>()
            .map_err(storage)
    }

    fn save(&mut self, plugin: &PluginRegistration) -> Result<()> {
        let state = String::from_utf8(encode(&MetadataDto::from(plugin.state.clone()))?)
            .map_err(|error| Error::Storage(error.to_string()))?;
        self.connection.execute("INSERT INTO plugins(name,executable,arguments,enabled,running,cursor,failures,retry_at,last_failure,subscriptions,capabilities,state) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12) ON CONFLICT(name) DO UPDATE SET enabled=excluded.enabled,running=excluded.running,cursor=excluded.cursor,failures=excluded.failures,retry_at=excluded.retry_at,last_failure=excluded.last_failure,subscriptions=excluded.subscriptions,capabilities=excluded.capabilities,state=excluded.state", params![plugin.name,plugin.executable,json(&plugin.arguments)?,plugin.enabled,plugin.running,plugin.cursor,plugin.failures,plugin.retry_at,plugin.last_failure,json(&plugin.subscriptions)?,json(&plugin.capabilities)?,state]).map_err(storage)?;

        Ok(())
    }

    fn count_managed(
        &mut self,
        owner: &str,
        states: &[anyq_queue_api::ManagedWorkState],
    ) -> Result<u64> {
        let states = states
            .iter()
            .map(|state| state.as_str())
            .collect::<Vec<_>>();
        let placeholders = vec!["?"; states.len()].join(",");
        let query = format!(
            "SELECT count(*) FROM managed_work WHERE owner=? AND state IN ({placeholders})"
        );
        let values = std::iter::once(owner).chain(states).collect::<Vec<_>>();
        let count: i64 = self
            .connection
            .query_row(&query, rusqlite::params_from_iter(values), |row| row.get(0))
            .map_err(storage)?;

        Ok(count as u64)
    }

    fn watermark(&mut self) -> Result<i64> {
        self.connection
            .query_row(
                "SELECT coalesce((SELECT seq FROM sqlite_sequence WHERE name='queue_events'),0)",
                [],
                |row| row.get(0),
            )
            .map_err(storage)
    }

    fn timers(&mut self, plugin: &str) -> Result<Vec<PluginTimer>> {
        let mut statement = self.connection.prepare("SELECT id,deadline,origin_sequence,payload FROM plugin_timers WHERE plugin=?1 ORDER BY deadline,id LIMIT 129").map_err(storage)?;
        statement
            .query_map([plugin], |row| {
                Ok(PluginTimer {
                    id: row.get(0)?,
                    deadline: row.get(1)?,
                    origin_sequence: row.get(2)?,
                    payload: metadata(&row.get::<_, String>(3)?).map_err(conversion)?,
                })
            })
            .map_err(storage)?
            .collect::<rusqlite::Result<_>>()
            .map_err(storage)
    }

    fn put_timer(&mut self, plugin: &str, timer: &PluginTimer) -> Result<()> {
        let payload = String::from_utf8(encode(&MetadataDto::from(timer.payload.clone()))?)
            .map_err(|error| Error::Storage(error.to_string()))?;
        self.connection.execute("INSERT INTO plugin_timers(plugin,id,deadline,origin_sequence,payload) VALUES (?1,?2,?3,?4,?5) ON CONFLICT(plugin,id) DO UPDATE SET deadline=excluded.deadline,origin_sequence=excluded.origin_sequence,payload=excluded.payload", params![plugin,timer.id,timer.deadline,timer.origin_sequence,payload]).map_err(storage)?;

        Ok(())
    }

    fn remove_timer(&mut self, plugin: &str, id: &str) -> Result<()> {
        self.connection
            .execute(
                "DELETE FROM plugin_timers WHERE plugin=?1 AND id=?2",
                params![plugin, id],
            )
            .map_err(storage)?;

        Ok(())
    }
}
