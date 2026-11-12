use anyq_plugin_api::PluginRegistration;
use anyq_plugin_wire::{CapabilityRequest, CapabilityResult, work_state};
use anyq_queue_api::*;

pub(crate) fn capability(
    plugin: &PluginRegistration,
    request: CapabilityRequest,
    read: &mut dyn GetTask,
    work: &mut crate::WorkCapabilities,
) -> Result<CapabilityResult> {
    if !plugin
        .capabilities
        .iter()
        .any(|allowed| allowed == request.capability())
    {
        return Err(Error::Invalid("plugin capability was not declared".into()));
    }
    if !plugin.enabled
        && !matches!(
            request,
            CapabilityRequest::ManagedReport { .. } | CapabilityRequest::ManagedList { .. }
        )
    {
        return Err(Error::Conflict("plugin is disabled".into()));
    }

    Ok(match request {
        CapabilityRequest::ReadTask { name } => CapabilityResult::Task {
            task: read.get(&name)?.into(),
        },
        CapabilityRequest::ReadState => CapabilityResult::State {
            state: plugin.state.clone().into(),
        },
        CapabilityRequest::Complete {
            action_id,
            task,
            force,
        } => {
            match work.complete_once.complete(CompleteTaskOnceRequest {
                owner: plugin.name.clone(),
                action_id,
                task: task.clone(),
                force,
            }) {
                Ok(task) => CapabilityResult::Task { task: task.into() },
                Err(Error::ManagedWorkPending) => {
                    if work
                        .begin_completion
                        .begin(BeginCompletionRequest { task, force })
                        .unwrap_or(false)
                    {
                        CapabilityResult::CompletionPending
                    } else {
                        return Err(Error::ManagedWorkPending);
                    }
                }
                Err(error) => return Err(error),
            }
        }
        CapabilityRequest::ManagedRegister {
            action_id,
            task,
            details,
        } => {
            let launch = work.register.register(RegisterManagedWorkRequest {
                details: details.try_into()?,
                owner: plugin.name.clone(),
                action_id,
                task,
            })?;
            CapabilityResult::ManagedLaunch {
                work: launch.work.into(),
                launch_allowed: launch.launch_allowed,
            }
        }
        CapabilityRequest::ManagedReport {
            action_id,
            state,
            details,
        } => CapabilityResult::ManagedWork {
            work: work
                .report
                .report(ReportManagedWorkRequest {
                    details: details.map(TryInto::try_into).transpose()?,
                    owner: plugin.name.clone(),
                    action_id,
                    state: work_state(&state)?,
                })?
                .into(),
        },
        CapabilityRequest::ManagedList { after } => {
            let page = work.list.list(ListManagedWorkRequest {
                owner: Some(plugin.name.clone()),
                task: None,
                after,
                limit: Some(1),
                states: Vec::new(),
            })?;
            let next = page.last().map(|work| work.action_id.clone());
            CapabilityResult::ManagedList {
                next,
                work: page.into_iter().map(Into::into).collect(),
            }
        }
    })
}
