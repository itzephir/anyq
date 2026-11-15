use crate::{
    execution::Execution, outbox::Outbox, phase::Phase, recovered_completion::RecoveredCompletion,
};
use anyq_queue_api::Result;
use std::{
    collections::BTreeMap,
    io::Write,
    time::{Duration, Instant},
};

pub(crate) struct PollCursor {
    action: String,
    prefer_recovery: bool,
}

impl PollCursor {
    pub fn new() -> Self {
        Self {
            action: String::new(),
            prefer_recovery: false,
        }
    }

    pub fn poll(
        &mut self,
        executions: &mut BTreeMap<String, Execution>,
        recovered: &mut BTreeMap<String, RecoveredCompletion>,
        stopping: bool,
        output: &mut Outbox<impl Write>,
    ) -> Result<()> {
        if !output.has_capacity() {
            return Ok(());
        }
        let recovery = recovered
            .iter()
            .filter(|(_, completion)| !stopping && completion.next_request <= Instant::now())
            .min_by_key(|(_, completion)| completion.next_request)
            .map(|(action, _)| action.clone());
        if self.prefer_recovery {
            if let Some(action) = &recovery {
                self.prefer_recovery = false;
                return recovered
                    .get_mut(action)
                    .expect("selected recovery exists")
                    .retry(output);
            }
        }

        let ready = |execution: &Execution| {
            execution.phase == Phase::Executing
                || execution.phase == Phase::Completing
                    && execution.last_report.elapsed() > Duration::from_millis(500)
        };
        let action = executions
            .iter()
            .filter(|(_, execution)| ready(execution))
            .find(|(action, _)| *action > &self.action)
            .or_else(|| executions.iter().find(|(_, execution)| ready(execution)))
            .map(|(action, _)| action.clone());
        if let Some(action) = action {
            self.action = action.clone();
            self.prefer_recovery = true;
            let execution = executions
                .get_mut(&action)
                .expect("selected execution exists");
            execution.poll(output)?;
            if execution.phase == Phase::Completing {
                execution.complete(output)?;
            }
        } else if let Some(action) = recovery {
            self.prefer_recovery = false;
            recovered
                .get_mut(&action)
                .expect("selected recovery exists")
                .retry(output)?;
        }

        Ok(())
    }
}
