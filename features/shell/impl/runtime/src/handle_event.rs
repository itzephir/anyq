use crate::{ProcessFactory, details, execution::Execution, phase::Phase, stdio};
use anyq_plugin_wire::{CapabilityRequest, PluginFrame};
use anyq_queue_api::{Metadata, MetadataValue, QueueEvent, QueueEventKind, Result, format_name};
use anyq_shell_api::Plan;
use std::{collections::BTreeMap, io::Write, time::Instant};

pub(crate) fn handle_event(
    event: QueueEvent,
    delivery_id: String,
    executions: &mut BTreeMap<String, Execution>,
    factory: &mut impl ProcessFactory,
    output: &mut impl Write,
) -> Result<()> {
    let prepared: Result<_> = (|| {
        if event.snapshot.kind != QueueEventKind::TaskRunning {
            return Ok(None);
        }
        let Some(task) = &event.snapshot.task else {
            return Ok(None);
        };
        let Some(plan) = Plan::from_task(task.metadata())? else {
            return Ok(None);
        };
        let plan = factory.prepare(plan, event.snapshot.directory.clone())?;
        let details = details::initial(&plan, &event.id())?;

        Ok(Some((task.name().to_string(), plan, details)))
    })();
    match prepared {
        Ok(Some((task, plan, details))) => {
            let action = format!("shell:{}", event.id());
            if executions.contains_key(&action) {
                return stdio::send(
                    output,
                    PluginFrame::Ack {
                        delivery_id,
                        state: None,
                        timers: vec![],
                    },
                );
            }
            stdio::send(
                output,
                PluginFrame::Request {
                    id: format!("register:{action}"),
                    request: CapabilityRequest::ManagedRegister {
                        action_id: action.clone(),
                        task: format_name(&task),
                        details: details.clone().into(),
                    },
                },
            )?;
            executions.insert(
                action.clone(),
                Execution {
                    action,
                    task,
                    delivery: Some(delivery_id),
                    plan,
                    details,
                    index: 0,
                    handle: None,
                    phase: Phase::Registering,
                    cancelled: false,
                    terminal: false,
                    last_report: Instant::now(),
                },
            );
            Ok(())
        }
        Ok(None) => stdio::send(
            output,
            PluginFrame::Ack {
                delivery_id,
                state: None,
                timers: vec![],
            },
        ),
        Err(error) => {
            let mut state = Metadata::default();
            state.set("last_error", MetadataValue::String(error.to_string()))?;
            state.set("error_event", MetadataValue::String(event.id()))?;
            stdio::send(
                output,
                PluginFrame::Ack {
                    delivery_id,
                    state: Some(state.into()),
                    timers: vec![],
                },
            )
        }
    }
}
