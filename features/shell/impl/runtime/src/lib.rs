mod checkpoint;
mod completion_result;
mod details;
mod execution;
mod handle_event;
mod handle_result;
mod launch_error;
mod managed_process;
mod phase;
mod prepare_plan;
mod recovered_completion;
mod spawn_process;
mod stdio;

pub use launch_error::LaunchError;
pub use managed_process::ManagedProcess;
pub use prepare_plan::PreparePlan;
pub use spawn_process::SpawnProcess;

use anyq_plugin_wire::{
    CapabilityRequest, CapabilityResult, HostFrame, PROTOCOL_VERSION, PluginFrame,
};
use anyq_queue_api::{Error, Result};
use phase::Phase;
use recovered_completion::RecoveredCompletion;
use std::{
    collections::BTreeMap,
    io::{BufRead, Write},
    sync::mpsc,
    time::{Duration, Instant},
};

pub fn run(
    mut input: impl BufRead + Send + 'static,
    output: &mut impl Write,
    factory: &mut (impl PreparePlan + SpawnProcess),
) -> Result<()> {
    let (sender, receiver) = mpsc::sync_channel(8);
    // This reader belongs to the executable mode's process lifetime. Its bounded
    // channel cannot allocate without limit; EOF/Stop cancels all native work.
    std::thread::spawn(move || {
        loop {
            match stdio::receive(&mut input) {
                Ok(Some(frame)) => {
                    if sender.send(Ok(frame)).is_err() {
                        return;
                    }
                }
                Ok(None) => return,
                Err(error) => {
                    let _ = sender.send(Err(error));
                    return;
                }
            }
        }
    });
    let mut executions = BTreeMap::new();
    let mut recovered: BTreeMap<String, RecoveredCompletion> = BTreeMap::new();
    let mut stopping: Option<Instant> = None;
    let mut handshaken = false;
    loop {
        match receiver.recv_timeout(Duration::from_millis(20)) {
            Ok(frame) => match frame? {
                HostFrame::Hello { version, .. } => {
                    if version != PROTOCOL_VERSION || handshaken {
                        return Err(Error::Invalid(
                            "incompatible shell protocol handshake".into(),
                        ));
                    }
                    handshaken = true;
                    stdio::send(
                        output,
                        PluginFrame::Hello {
                            version: PROTOCOL_VERSION,
                            subscriptions: vec!["task.running".into()],
                            capabilities: vec![
                                "read".into(),
                                "complete".into(),
                                "managed".into(),
                                "state".into(),
                            ],
                        },
                    )?;
                    stdio::send(
                        output,
                        PluginFrame::Request {
                            id: "recovery".into(),
                            request: CapabilityRequest::ManagedList { after: None },
                        },
                    )?;
                }
                _ if !handshaken => {
                    return Err(Error::Invalid("shell handshake is required".into()));
                }
                HostFrame::Event { event, delivery_id } => handle_event::handle_event(
                    event.try_into()?,
                    delivery_id,
                    &mut executions,
                    factory,
                    output,
                )?,
                HostFrame::Cancel { work } => {
                    if let Some(execution) = executions.get_mut(&work.action_id) {
                        execution.cancel()?;
                    }
                }
                HostFrame::Result {
                    id,
                    result: CapabilityResult::ManagedList { work, next },
                } if id == "recovery" => {
                    for work in work {
                        if let Some(completion) = RecoveredCompletion::from_work(work)? {
                            recovered.insert(completion.action.clone(), completion);
                        }
                    }
                    if let Some(after) = next {
                        stdio::send(
                            output,
                            PluginFrame::Request {
                                id: "recovery".into(),
                                request: CapabilityRequest::ManagedList { after: Some(after) },
                            },
                        )?;
                    }
                }
                HostFrame::Result { id, result }
                    if id.starts_with("recovered:") || id.starts_with("recovered-read:") =>
                {
                    if let Some((_, action)) = id.split_once(':') {
                        let finished = match recovered.get(action) {
                            Some(completion) => completion.accept(&id, result, output)?,
                            None => false,
                        };
                        if finished {
                            recovered.remove(action);
                        }
                    }
                }
                HostFrame::Result { id, result } => {
                    handle_result::handle_result(&id, result, &mut executions, factory, output)?
                }
                HostFrame::Stop => {
                    stopping.get_or_insert_with(Instant::now);
                }
                HostFrame::Timer { .. } => {
                    return Err(Error::Invalid("shell does not schedule timers".into()));
                }
            },
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                stopping.get_or_insert_with(Instant::now);
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
        for execution in executions.values_mut() {
            if stopping.is_some() {
                execution.cancel()?;
            }
            execution.poll(output)?;
            if execution.phase == Phase::Completing
                && execution.last_report.elapsed() > Duration::from_millis(500)
            {
                execution.complete(output)?;
            }
            if execution.phase == Phase::Finished {
                checkpoint::checkpoint("shell.execution-finished")?;
            }
        }
        executions.retain(|_, execution| {
            execution.phase != Phase::Finished && execution.phase != Phase::Uncertain
        });
        // Oldest request first: bound retries without starving a large recovery set.
        if stopping.is_none() {
            if let Some(completion) = recovered
                .values_mut()
                .min_by_key(|completion| completion.next_request)
            {
                completion.retry(output)?;
            }
        }
        if stopping
            .is_some_and(|at| executions.is_empty() || at.elapsed() >= Duration::from_secs(2))
        {
            return Ok(());
        }
    }
}

#[cfg(test)]
mod launch_failure;
#[cfg(test)]
mod tests;
