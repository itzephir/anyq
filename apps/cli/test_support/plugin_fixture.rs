//! Native test executable, excluded from ordinary builds and release archives.
use anyq_plugin_wire::{
    CapabilityRequest, CapabilityResult, HostFrame, PluginFrame, TimerDto, TimerUpdateDto, decode,
    encode,
};
use anyq_queue_api::{Metadata, MetadataValue};
use std::{
    fs::{self, OpenOptions},
    io::{self, BufRead, Write},
    path::PathBuf,
};

fn send(frame: PluginFrame) {
    let mut stdout = io::stdout().lock();
    stdout.write_all(&encode(&frame).unwrap()).unwrap();
    stdout.write_all(b"\n").unwrap();
    stdout.flush().unwrap();
}

fn main() {
    let arguments: Vec<_> = std::env::args().collect();
    let mode = &arguments[1];
    let log = PathBuf::from(&arguments[2]);
    let marker = log.with_extension("marker");
    let mut pending = None;

    for line in io::stdin().lock().lines() {
        let frame = match line {
            Ok(line) => decode::<HostFrame>(line.as_bytes()).unwrap(),
            Err(_) => return,
        };
        match frame {
            HostFrame::Hello { .. } => {
                if mode == "no-handshake" {
                    continue;
                }
                if mode == "flood" {
                    let bytes = vec![b'x'; 17 * 1024 * 1024];
                    let _ = io::stdout().write_all(&bytes);
                    let _ = io::stdout().flush();
                    continue;
                }
                send(PluginFrame::Hello {
                    version: if mode == "incompatible" { 999 } else { 1 },
                    subscriptions: vec!["task.running".into()],
                    capabilities: vec![
                        "read".into(),
                        "complete".into(),
                        "state".into(),
                        "timers".into(),
                        "managed".into(),
                    ],
                });
            }
            HostFrame::Event { delivery_id, event } => {
                let mut file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&log)
                    .unwrap();
                writeln!(
                    file,
                    "{}",
                    serde_json::json!({"id":delivery_id,"task":event.task.as_ref().unwrap().name})
                )
                .unwrap();
                if mode == "hang" {
                    continue;
                }
                if mode == "bad-state" {
                    let key = format!("SECRET-{}", "x".repeat(30000));
                    let frame = serde_json::json!({"type":"ack","delivery_id":delivery_id,"state":{"object":[[key.clone(),{"string":"sensitive"}],[key,{"bool":true}]]},"timers":[]});
                    println!("{frame}");
                    io::stdout().flush().unwrap();
                    continue;
                }
                if mode == "crash-once" && !marker.exists() {
                    fs::write(&marker, b"crashed").unwrap();
                    std::process::exit(42);
                }
                if mode == "complete-lost" {
                    send(PluginFrame::Request {
                        id: "complete".into(),
                        request: CapabilityRequest::Complete {
                            action_id: format!("complete:{delivery_id}"),
                            task: anyq_queue_api::format_name(&event.task.as_ref().unwrap().name),
                            force: false,
                        },
                    });
                    pending = Some((delivery_id, event));
                } else if mode == "managed" || mode == "uncertain" {
                    send(PluginFrame::Request {
                        id: "register".into(),
                        request: CapabilityRequest::ManagedRegister {
                            action_id: format!("work:{delivery_id}"),
                            task: anyq_queue_api::format_name(&event.task.as_ref().unwrap().name),
                        },
                    });
                    pending = Some((delivery_id, event));
                } else if mode == "timer" {
                    let mut state = Metadata::default();
                    state.set("scheduled", MetadataValue::Bool(true)).unwrap();
                    send(PluginFrame::Ack {
                        delivery_id,
                        state: Some(state.into()),
                        timers: vec![TimerUpdateDto::Put {
                            timer: TimerDto {
                                id: "deadline".into(),
                                deadline: event.occurred_at + 700,
                                payload: Metadata::default().into(),
                            },
                        }],
                    });
                } else {
                    send(PluginFrame::Ack {
                        delivery_id,
                        state: None,
                        timers: vec![],
                    });
                }
            }
            HostFrame::Result {
                id,
                result:
                    CapabilityResult::ManagedLaunch {
                        work,
                        launch_allowed,
                    },
            } if id == "register" => {
                if launch_allowed {
                    fs::write(&marker, b"started").unwrap();
                    send(PluginFrame::Request {
                        id: "active".into(),
                        request: CapabilityRequest::ManagedReport {
                            action_id: work.action_id,
                            state: "active".into(),
                        },
                    });
                }
                let (delivery_id, _) = pending.take().unwrap();
                send(PluginFrame::Ack {
                    delivery_id,
                    state: None,
                    timers: vec![],
                });
            }
            HostFrame::Result { id, result } if id == "complete" => {
                if matches!(result, CapabilityResult::Task { .. }) {
                    if !marker.exists() {
                        fs::write(&marker, b"committed").unwrap();
                        std::process::exit(43);
                    }
                    let (delivery_id, _) = pending.take().unwrap();
                    send(PluginFrame::Ack {
                        delivery_id,
                        state: None,
                        timers: vec![],
                    });
                }
            }
            HostFrame::Cancel { work } => {
                fs::write(
                    &marker,
                    if mode == "uncertain" {
                        b"uncertain".as_slice()
                    } else {
                        b"stopped".as_slice()
                    },
                )
                .unwrap();
                send(PluginFrame::Request {
                    id: "stop".into(),
                    request: CapabilityRequest::ManagedReport {
                        action_id: work.action_id,
                        state: if mode == "uncertain" {
                            "uncertain"
                        } else {
                            "stopped"
                        }
                        .into(),
                    },
                });
            }
            HostFrame::Timer { delivery_id, timer } => {
                let mut file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&log)
                    .unwrap();
                writeln!(
                    file,
                    "{}",
                    serde_json::json!({"timer":timer.id,"deadline":timer.deadline})
                )
                .unwrap();
                send(PluginFrame::Ack {
                    delivery_id,
                    state: None,
                    timers: vec![],
                });
            }
            HostFrame::Stop => return,
            _ => {}
        }
    }
}
