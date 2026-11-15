use crate::outbox::Outbox;
use anyq_plugin_wire::{CapabilityRequest, CapabilityResult, ManagedWorkDto, PluginFrame};
use anyq_queue_api::{Metadata, MetadataValue, Result, format_name};
use std::{
    io::Write,
    time::{Duration, Instant},
};

pub(crate) struct RecoveredCompletion {
    pub action: String,
    task: String,
    pub next_request: Instant,
}

impl RecoveredCompletion {
    pub fn from_work(work: ManagedWorkDto) -> Result<Option<Self>> {
        let details: Metadata = work.details.try_into()?;
        if work.state != "stopped"
            || details.get(Some("autocomplete")) != Ok(&MetadataValue::Bool(true))
            || !matches!(details.get(Some("status")), Ok(MetadataValue::String(status)) if status == "exited" || status == "launch-failed")
        {
            return Ok(None);
        }

        Ok(Some(Self {
            action: work.action_id,
            task: work.task,
            next_request: Instant::now(),
        }))
    }

    pub fn retry(&mut self, output: &mut Outbox<impl Write>) -> Result<()> {
        if Instant::now() < self.next_request {
            return Ok(());
        }

        output.send(PluginFrame::Request {
            id: format!("recovered:{}", self.action),
            request: CapabilityRequest::Complete {
                action_id: format!("complete:{}", self.action),
                task: format_name(&self.task),
                force: false,
            },
        })?;
        self.next_request = Instant::now() + Duration::from_millis(500);

        Ok(())
    }

    pub fn accept(
        &self,
        id: &str,
        result: CapabilityResult,
        output: &mut Outbox<impl Write>,
    ) -> Result<bool> {
        crate::completion_result::accept(id, &self.task, result, output)
    }
}
