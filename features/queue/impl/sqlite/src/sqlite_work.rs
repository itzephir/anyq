use crate::storage_error::storage;
use anyq_queue_api::{
    CompletionReceipt, Error, ManagedWork, ManagedWorkFilter, ManagedWorkState, Result,
};
use anyq_queue_dispatch::ports::WorkRecords;
use anyq_queue_wire::TaskDto;
use rusqlite::{Connection, OptionalExtension, params};
use serde::Deserialize;

pub(crate) struct SqliteWork<'a> {
    pub connection: &'a Connection,
}

impl WorkRecords for SqliteWork<'_> {
    fn managed(&mut self, filter: &ManagedWorkFilter) -> Result<Vec<ManagedWork>> {
        let mut statement = self.connection.prepare("SELECT owner,action_id,task_name,state,cancellation_requested,details FROM managed_work WHERE (?1 IS NULL OR owner=?1) AND (?2 IS NULL OR task_name=?2) AND (?3 IS NULL OR action_id=?3) AND (?4 IS NULL OR action_id>?4) AND (?6 OR state IN (?7,?8,?9,?10)) ORDER BY owner,action_id LIMIT ?5").map_err(storage)?;
        let rows = statement
            .query_map(
                params![
                    filter.owner,
                    filter.task,
                    filter.action,
                    filter.after,
                    filter.limit.map(|limit| limit as i64).unwrap_or(-1),
                    filter.states.is_empty(),
                    filter.states.first().map(|state| state.as_str()),
                    filter.states.get(1).map(|state| state.as_str()),
                    filter.states.get(2).map(|state| state.as_str()),
                    filter.states.get(3).map(|state| state.as_str())
                ],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, bool>(4)?,
                        row.get::<_, String>(5)?,
                    ))
                },
            )
            .map_err(storage)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(storage)?;

        rows.into_iter()
            .map(
                |(owner, action_id, task, state, cancellation_requested, details)| {
                    let state = match state.as_str() {
                        "launch-pending" => ManagedWorkState::LaunchPending,
                        "active" => ManagedWorkState::Active,
                        "stopped" => ManagedWorkState::Stopped,
                        "uncertain" => ManagedWorkState::Uncertain,
                        _ => return Err(Error::Storage("unknown managed work state".into())),
                    };

                    let mut decoder = serde_json::Deserializer::from_str(&details);
                    decoder.disable_recursion_limit();
                    let details = anyq_queue_wire::MetadataDto::deserialize(&mut decoder)
                        .map_err(|error| Error::Storage(error.to_string()))?;
                    decoder
                        .end()
                        .map_err(|error| Error::Storage(error.to_string()))?;

                    Ok(ManagedWork {
                        details: details.try_into()?,
                        owner,
                        action_id,
                        task,
                        state,
                        cancellation_requested,
                    })
                },
            )
            .collect()
    }

    fn save_managed(&mut self, work: &ManagedWork) -> Result<()> {
        self.connection.execute("INSERT INTO managed_work(owner,action_id,task_name,state,cancellation_requested,details) VALUES (?1,?2,?3,?4,?5,?6) ON CONFLICT(owner,action_id) DO UPDATE SET state=excluded.state,cancellation_requested=excluded.cancellation_requested,details=excluded.details", params![work.owner,work.action_id,work.task,work.state.as_str(),work.cancellation_requested,serde_json::to_string(&anyq_queue_wire::MetadataDto::from(work.details.clone())).map_err(|error| Error::Storage(error.to_string()))?]).map_err(storage)?;

        Ok(())
    }

    fn completion_requested(&mut self, task: &str) -> Result<bool> {
        self.connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM completion_intents WHERE task_name=?1)",
                [task],
                |row| row.get(0),
            )
            .map_err(storage)
    }

    fn request_completion(&mut self, task: &str) -> Result<()> {
        self.connection
            .execute(
                "INSERT OR IGNORE INTO completion_intents(task_name) VALUES (?1)",
                [task],
            )
            .map_err(storage)?;
        self.connection.execute("UPDATE managed_work SET cancellation_requested=1 WHERE task_name=?1 AND state!='stopped'", [task]).map_err(storage)?;

        Ok(())
    }

    fn receipt(&mut self, owner: &str, action_id: &str) -> Result<Option<CompletionReceipt>> {
        let row = self.connection.query_row("SELECT task_name,force,result FROM completion_receipts WHERE owner=?1 AND action_id=?2", params![owner,action_id], |row| Ok((row.get::<_,String>(0)?,row.get::<_,bool>(1)?,row.get::<_,String>(2)?)))
            .optional().map_err(storage)?;

        row.map(|(task, force, result)| {
            let mut decoder = serde_json::Deserializer::from_str(&result);
            decoder.disable_recursion_limit();
            let result = TaskDto::deserialize(&mut decoder)
                .map_err(|error| Error::Storage(error.to_string()))?;
            decoder
                .end()
                .map_err(|error| Error::Storage(error.to_string()))?;

            Ok(CompletionReceipt {
                task,
                force,
                result: result.try_into()?,
            })
        })
        .transpose()
    }

    fn save_receipt(
        &mut self,
        owner: &str,
        action_id: &str,
        receipt: &CompletionReceipt,
    ) -> Result<()> {
        let result = serde_json::to_string(&TaskDto::from(receipt.result.clone()))
            .map_err(|error| Error::Storage(error.to_string()))?;
        self.connection.execute("INSERT INTO completion_receipts(owner,action_id,task_name,force,result) VALUES (?1,?2,?3,?4,?5)", params![owner,action_id,receipt.task,receipt.force,result]).map_err(storage)?;

        Ok(())
    }
}
