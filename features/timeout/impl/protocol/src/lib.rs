mod pending;
mod stdio;

use anyq_plugin_wire::{
    CapabilityRequest, CapabilityResult, HostFrame, PROTOCOL_VERSION, PluginFrame, TimerDto,
    TimerUpdateDto,
};
use anyq_queue_api::{Error, Metadata, MetadataValue, QueueEvent, Result, format_name};
use anyq_timeout_api::Deadline;
use pending::Pending;
use std::io::{BufRead, Write};

fn acknowledge(
    output: &mut impl Write,
    delivery_id: String,
    state: &Metadata,
    timers: Vec<TimerUpdateDto>,
) -> Result<()> {
    stdio::send(
        output,
        PluginFrame::Ack {
            delivery_id,
            state: Some(state.clone().into()),
            timers,
        },
    )
}

fn retire(state: &mut Metadata, plan: &Deadline) -> Result<()> {
    state.unset(&format!("active.{}", plan.event_id))?;
    state.set("last_resolved", MetadataValue::String(plan.task.clone()))
}

pub fn run(input: &mut impl BufRead, output: &mut impl Write) -> Result<()> {
    let mut state = Metadata::default();
    let mut pending: Option<Pending> = None;
    while let Some(frame) = stdio::receive(input)? {
        match frame {
            HostFrame::Hello {
                version,
                state: saved,
                ..
            } => {
                if version != PROTOCOL_VERSION {
                    return Err(Error::Invalid("incompatible plugin protocol".into()));
                }
                state = saved.try_into()?;
                stdio::send(
                    output,
                    PluginFrame::Hello {
                        version: PROTOCOL_VERSION,
                        subscriptions: vec!["task.running".into(), "task.completed".into()],
                        capabilities: vec![
                            "read".into(),
                            "complete".into(),
                            "state".into(),
                            "timers".into(),
                        ],
                    },
                )?;
            }
            HostFrame::Event { delivery_id, event } => {
                let event: QueueEvent = event.try_into()?;
                let mut timers = Vec::new();
                if event.snapshot.kind == anyq_queue_api::QueueEventKind::TaskRunning {
                    match anyq_timeout_api::resolve(&event) {
                        Ok(Some(plan)) => {
                            let payload = plan.metadata()?;
                            state.set(
                                &format!("active.{}", plan.event_id),
                                payload.clone().into_value(),
                            )?;
                            timers.push(TimerUpdateDto::Put {
                                timer: TimerDto {
                                    id: plan.event_id.clone(),
                                    deadline: plan.deadline,
                                    payload: payload.into(),
                                },
                            });
                        }
                        Ok(None) => {}
                        Err(error) => {
                            state.set("last_error", MetadataValue::String(error.to_string()))?;
                            state.set("error_event", MetadataValue::String(event.id()))?;
                        }
                    }
                } else if let Some(task) = event.snapshot.task {
                    if let Ok(MetadataValue::Object(active)) = state.get(Some("active")) {
                        let plans = active
                            .values()
                            .cloned()
                            .map(|value| Deadline::restore(Metadata::new(value)?))
                            .collect::<Result<Vec<_>>>()?;
                        for plan in plans.into_iter().filter(|plan| plan.task == task.name()) {
                            retire(&mut state, &plan)?;
                            timers.push(TimerUpdateDto::Remove { id: plan.event_id });
                        }
                    }
                }
                acknowledge(output, delivery_id, &state, timers)?;
            }
            HostFrame::Timer { delivery_id, timer } => {
                let plan = Deadline::restore(timer.payload.try_into()?)?;
                stdio::send(
                    output,
                    PluginFrame::Request {
                        id: "read".into(),
                        request: CapabilityRequest::ReadTask {
                            name: format_name(&plan.task),
                        },
                    },
                )?;
                pending = Some(Pending { delivery_id, plan });
            }
            HostFrame::Result { id, result } => {
                let Some(current) = pending.take() else {
                    return Err(Error::Invalid("unexpected timeout response".into()));
                };
                if id == "read"
                    && matches!(&result, CapabilityResult::Task { task } if task.state == "running")
                {
                    stdio::send(
                        output,
                        PluginFrame::Request {
                            id: "complete".into(),
                            request: CapabilityRequest::Complete {
                                action_id: format!("timeout:{}", current.plan.event_id),
                                task: format_name(&current.plan.task),
                                force: false,
                            },
                        },
                    )?;
                    pending = Some(current);
                    continue;
                }
                let resolved =
                    matches!(&result, CapabilityResult::Task { task } if task.state == "completed");
                let timers = if resolved {
                    retire(&mut state, &current.plan)?;
                    Vec::new()
                } else {
                    state.set("last_error", MetadataValue::String(match result { CapabilityResult::CompletionPending => "timeout pending: managed work must confirm stopped".into(), CapabilityResult::Error { message, .. } => message, _ => "timeout requires a running task; queued/registered tasks are never forced".into() }))?;
                    vec![TimerUpdateDto::Put {
                        timer: TimerDto {
                            id: current.plan.event_id.clone(),
                            deadline: current.plan.deadline,
                            payload: current.plan.metadata()?.into(),
                        },
                    }]
                };
                acknowledge(output, current.delivery_id, &state, timers)?;
            }
            HostFrame::Stop => return Ok(()),
            HostFrame::Cancel { .. } => {
                return Err(Error::Invalid("timeout does not own managed work".into()));
            }
        }
    }

    Ok(())
}
