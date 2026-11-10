mod checkpoint;
mod details;
mod execution;
mod handle_event;
mod handle_result;
mod launch_error;
mod managed_process;
mod phase;
mod prepare_plan;
mod spawn_process;
mod stdio;

pub use launch_error::LaunchError;
pub use managed_process::ManagedProcess;
pub use prepare_plan::PreparePlan;
pub use spawn_process::SpawnProcess;

use anyq_plugin_wire::{
    CapabilityRequest, CapabilityResult, HostFrame, PROTOCOL_VERSION, PluginFrame,
};
use anyq_queue_api::{Error, Metadata, MetadataValue, Result, format_name};
use phase::Phase;
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
                            request: CapabilityRequest::ManagedList,
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
                    result: CapabilityResult::ManagedList { work },
                } if id == "recovery" => {
                    for work in work {
                        let details: Metadata = work.details.try_into()?;
                        if work.state == "stopped"
                            && details.get(Some("autocomplete")) == Ok(&MetadataValue::Bool(true))
                            && matches!(details.get(Some("status")), Ok(MetadataValue::String(status)) if status == "exited" || status == "launch-failed")
                        {
                            stdio::send(
                                output,
                                PluginFrame::Request {
                                    id: format!("recovered:{}", work.action_id),
                                    request: CapabilityRequest::Complete {
                                        action_id: format!("complete:{}", work.action_id),
                                        task: format_name(&work.task),
                                        force: false,
                                    },
                                },
                            )?;
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
        }
        executions.retain(|_, execution| {
            execution.phase != Phase::Finished && execution.phase != Phase::Uncertain
        });
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
