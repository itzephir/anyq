mod checkpoint;
mod completion_result;
mod details;
mod execution;
mod handle_event;
mod handle_result;
mod launch_error;
mod managed_process;
mod outbox;
mod phase;
mod poll_cursor;
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
use outbox::Outbox;
use phase::Phase;
use poll_cursor::PollCursor;
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
    let mut output = Outbox::new(output);
    let mut cursor = PollCursor::new();
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
            Ok(frame) => {
                let frame = frame?;
                if let HostFrame::Result { id, .. } = &frame {
                    output.acknowledge(id);
                }
                match frame {
                    HostFrame::Hello { version, .. } => {
                        if version != PROTOCOL_VERSION || handshaken {
                            return Err(Error::Invalid(
                                "incompatible shell protocol handshake".into(),
                            ));
                        }
                        handshaken = true;
                        output.send(PluginFrame::Hello {
                            version: PROTOCOL_VERSION,
                            subscriptions: vec!["task.running".into()],
                            capabilities: vec![
                                "read".into(),
                                "complete".into(),
                                "managed".into(),
                                "state".into(),
                            ],
                        })?;
                        output.send(PluginFrame::Request {
                            id: "recovery".into(),
                            request: CapabilityRequest::ManagedList { after: None },
                        })?;
                    }
                    _ if !handshaken => {
                        return Err(Error::Invalid("shell handshake is required".into()));
                    }
                    HostFrame::Event { event, delivery_id } => handle_event::handle_event(
                        event.try_into()?,
                        delivery_id,
                        &mut executions,
                        factory,
                        &mut output,
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
                            output.send(PluginFrame::Request {
                                id: "recovery".into(),
                                request: CapabilityRequest::ManagedList { after: Some(after) },
                            })?;
                        }
                    }
                    HostFrame::Result { id, result }
                        if id.starts_with("recovered:") || id.starts_with("recovered-read:") =>
                    {
                        if let Some((_, action)) = id.split_once(':') {
                            let finished = match recovered.get(action) {
                                Some(completion) => completion.accept(&id, result, &mut output)?,
                                None => false,
                            };
                            if finished {
                                recovered.remove(action);
                            }
                        }
                    }
                    HostFrame::Result { id, result } => handle_result::handle_result(
                        &id,
                        result,
                        &mut executions,
                        factory,
                        &mut output,
                    )?,
                    HostFrame::Stop => {
                        stopping.get_or_insert_with(Instant::now);
                    }
                    HostFrame::Timer { .. } => {
                        return Err(Error::Invalid("shell does not schedule timers".into()));
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                stopping.get_or_insert_with(Instant::now);
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
        if stopping.is_some() {
            for execution in executions.values_mut() {
                execution.cancel()?;
            }
        }
        output.flush()?;
        cursor.poll(
            &mut executions,
            &mut recovered,
            stopping.is_some(),
            &mut output,
        )?;
        for execution in executions.values() {
            if execution.phase == Phase::Finished {
                checkpoint::checkpoint("shell.execution-finished")?;
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
