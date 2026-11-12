use crate::{details, execution::Execution, launch_failure::LaunchFailure, phase::Phase};
use anyq_plugin_wire::{CapabilityRequest, PluginFrame, decode};
use anyq_queue_api::ExecutionDirectory;
use anyq_shell_api::{Command, PreparedPlan};
use std::{collections::BTreeMap, time::Instant};

#[test]
fn uncertain_launch_cannot_report_stopped_or_request_autocomplete() {
    for uncertain in [false, true] {
        let plan = PreparedPlan {
            commands: vec![Command {
                program: "unused".into(),
                arguments: Vec::new(),
                raw_shell: None,
                directory: ExecutionDirectory::Unix(b"/".to_vec()),
                environment: BTreeMap::new(),
            }],
            autocomplete: true,
        };
        let details = details::initial(&plan, "db:1").unwrap();
        let mut execution = Execution {
            action: "shell:db:1".into(),
            task: "job".into(),
            delivery: None,
            plan,
            details,
            index: 0,
            handle: None,
            phase: Phase::Registering,
            cancelled: false,
            terminal: false,
            last_report: Instant::now(),
        };
        let mut output = Vec::new();

        execution
            .launch(&mut LaunchFailure { uncertain }, &mut output)
            .unwrap();

        let frame: PluginFrame = decode(&output).unwrap();
        let PluginFrame::Request {
            request: CapabilityRequest::ManagedReport { state, .. },
            ..
        } = frame
        else {
            panic!("launch failure must await a durable managed report")
        };
        assert_eq!(state, if uncertain { "uncertain" } else { "stopped" });
        assert_eq!(execution.phase, Phase::Reporting);
    }
}
