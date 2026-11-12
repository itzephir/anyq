use crate::{ManagedProcess, SpawnProcess, details, phase::Phase, stdio};
use anyq_plugin_wire::{CapabilityRequest, PluginFrame};
use anyq_queue_api::{Metadata, MetadataValue, Result, format_name};
use anyq_shell_api::{EntryOutcome, EntryOutput, PreparedPlan};
use std::{io::Write, time::Instant};

pub(crate) struct Execution {
    pub action: String,
    pub task: String,
    pub delivery: Option<String>,
    pub plan: PreparedPlan,
    pub details: Metadata,
    pub index: usize,
    pub handle: Option<Box<dyn ManagedProcess>>,
    pub phase: Phase,
    pub cancelled: bool,
    pub terminal: bool,
    pub last_report: Instant,
}

impl Execution {
    pub fn report(&mut self, output: &mut impl Write, state: &str) -> Result<()> {
        stdio::send(
            output,
            PluginFrame::Request {
                id: format!("report:{}", self.action),
                request: CapabilityRequest::ManagedReport {
                    action_id: self.action.clone(),
                    state: state.into(),
                    details: Some(self.details.clone().into()),
                },
            },
        )?;
        self.phase = Phase::Reporting;
        self.last_report = Instant::now();

        Ok(())
    }

    pub fn complete(&mut self, output: &mut impl Write) -> Result<()> {
        stdio::send(
            output,
            PluginFrame::Request {
                id: format!("complete:{}", self.action),
                request: CapabilityRequest::Complete {
                    action_id: format!("complete:{}", self.action),
                    task: format_name(&self.task),
                    force: false,
                },
            },
        )?;
        self.phase = Phase::Completing;
        self.last_report = Instant::now();

        Ok(())
    }

    pub fn cancel(&mut self) -> Result<()> {
        self.cancelled = true;
        if let Some(handle) = &mut self.handle {
            handle.cancel()?;
        }

        Ok(())
    }

    pub fn launch(
        &mut self,
        factory: &mut impl SpawnProcess,
        output: &mut impl Write,
    ) -> Result<()> {
        if self.cancelled {
            self.terminal = true;
            self.details
                .set("status", MetadataValue::String("cancelled".into()))?;
            return self.report(output, "stopped");
        }
        crate::checkpoint::checkpoint("shell.before-spawn")?;
        match factory.spawn(&self.plan.commands[self.index]) {
            Ok(handle) => {
                self.handle = Some(handle);
                crate::checkpoint::checkpoint("shell.after-spawn")?;
                self.details
                    .set("status", MetadataValue::String("running".into()))?;
                self.report(output, "active")
            }
            Err(crate::LaunchError::Uncertain(message)) => {
                self.details
                    .set("status", MetadataValue::String("uncertain".into()))?;
                self.details.set("error", MetadataValue::String(message))?;
                self.report(output, "uncertain")
            }
            Err(crate::LaunchError::NotStarted(error)) => {
                self.terminal = true;
                self.details
                    .set("status", MetadataValue::String("launch-failed".into()))?;
                self.details
                    .set("error", MetadataValue::String(error.to_string()))?;
                details::outcome(
                    &mut self.details,
                    self.index,
                    EntryOutcome {
                        code: None,
                        success: false,
                        output: EntryOutput::default(),
                    },
                )?;
                self.report(output, "stopped")
            }
        }
    }

    pub fn poll(&mut self, output: &mut impl Write) -> Result<()> {
        if self.phase != Phase::Executing {
            return Ok(());
        }
        let Some(handle) = &mut self.handle else {
            return Ok(());
        };
        match handle.poll() {
            Ok(Some(outcome)) => {
                let success = outcome.success;
                details::outcome(&mut self.details, self.index, outcome)?;
                self.handle = None;
                self.index += 1;
                self.terminal =
                    self.cancelled || !success || self.index == self.plan.commands.len();
                self.details.set(
                    "status",
                    MetadataValue::String(
                        if self.cancelled {
                            "cancelled"
                        } else if self.terminal {
                            "exited"
                        } else {
                            "between-entries"
                        }
                        .into(),
                    ),
                )?;
                self.report(output, if self.terminal { "stopped" } else { "active" })?;
            }
            Ok(None) => {
                if self.last_report.elapsed() >= std::time::Duration::from_millis(500) {
                    self.details
                        .set("live_output", details::output(handle.output()))?;
                    self.report(output, "active")?;
                }
            }
            Err(error) => {
                self.details
                    .set("status", MetadataValue::String("uncertain".into()))?;
                self.details
                    .set("error", MetadataValue::String(error.to_string()))?;
                let _ = handle.cancel();
                self.report(output, "uncertain")?;
            }
        }

        Ok(())
    }
}
