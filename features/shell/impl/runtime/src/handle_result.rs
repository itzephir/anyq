use crate::{SpawnProcess, execution::Execution, phase::Phase, stdio};
use anyq_plugin_wire::{CapabilityResult, PluginFrame};
use anyq_queue_api::Result;
use std::{collections::BTreeMap, io::Write};

pub(crate) fn handle_result(
    id: &str,
    result: CapabilityResult,
    executions: &mut BTreeMap<String, Execution>,
    factory: &mut impl SpawnProcess,
    output: &mut impl Write,
) -> Result<()> {
    let Some((operation, action)) = id.split_once(':') else {
        return Ok(());
    };
    let Some(execution) = executions.get_mut(action) else {
        return Ok(());
    };
    match (operation, result) {
        (
            "register",
            CapabilityResult::ManagedLaunch {
                work,
                launch_allowed,
            },
        ) => {
            if let Some(delivery_id) = execution.delivery.take() {
                stdio::send(
                    output,
                    PluginFrame::Ack {
                        delivery_id,
                        state: None,
                        timers: Vec::new(),
                    },
                )?;
            }
            if launch_allowed {
                if work.cancellation_requested {
                    execution.cancel()?;
                }
                execution.launch(factory, output)?;
            } else {
                // A durable launch intent is never permission to run again.
                execution.phase = Phase::Uncertain;
            }
        }
        ("report", CapabilityResult::ManagedWork { work }) => {
            if work.cancellation_requested {
                execution.cancel()?;
            }
            if work.state == "uncertain" {
                execution.phase = Phase::Uncertain;
            } else if execution.terminal && work.state == "stopped" {
                if execution.plan.autocomplete && !execution.cancelled {
                    execution.complete(output)?;
                } else {
                    execution.phase = Phase::Finished;
                }
            } else if execution.handle.is_none() {
                execution.launch(factory, output)?;
            } else {
                execution.phase = Phase::Executing;
            }
        }
        ("complete", CapabilityResult::Task { task }) if task.state == "completed" => {
            execution.phase = Phase::Finished
        }
        ("complete", _) => {
            // Retry with the same receipt ID. A manual completion may have won.
            execution.phase = Phase::Completing;
        }
        (_, CapabilityResult::Error { .. }) => {
            let _ = execution.cancel();
            execution.phase = Phase::Uncertain;
        }
        _ => {}
    }

    Ok(())
}
