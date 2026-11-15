use crate::stdio;
use anyq_plugin_wire::{CapabilityRequest, CapabilityResult, PluginFrame};
use anyq_queue_api::{Result, format_name};
use std::io::Write;

pub(crate) fn accept(
    id: &str,
    task: &str,
    result: CapabilityResult,
    output: &mut impl Write,
) -> Result<bool> {
    match result {
        CapabilityResult::Task { task } if task.state == "completed" => Ok(true),
        CapabilityResult::Error { category, .. } if category == "conflict" => {
            let Some((operation, action)) = id
                .split_once(':')
                .filter(|(operation, _)| matches!(*operation, "complete" | "recovered"))
            else {
                return Ok(false);
            };

            // Another caller may have completed without this action's receipt.
            stdio::send(
                output,
                PluginFrame::Request {
                    id: format!("{operation}-read:{action}"),
                    request: CapabilityRequest::ReadTask {
                        name: format_name(task),
                    },
                },
            )?;

            Ok(false)
        }
        _ => Ok(false),
    }
}
