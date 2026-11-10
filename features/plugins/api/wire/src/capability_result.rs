use crate::ManagedWorkDto;
use anyq_queue_wire::{MetadataDto, TaskDto};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum CapabilityResult {
    Task {
        task: TaskDto,
    },
    CompletionPending,
    State {
        state: MetadataDto,
    },
    ManagedLaunch {
        work: ManagedWorkDto,
        launch_allowed: bool,
    },
    ManagedWork {
        work: ManagedWorkDto,
    },
    ManagedList {
        work: Vec<ManagedWorkDto>,
        next: Option<String>,
    },
    Error {
        category: String,
        message: String,
    },
}

impl From<anyq_queue_api::Error> for CapabilityResult {
    fn from(error: anyq_queue_api::Error) -> Self {
        let category = match &error {
            anyq_queue_api::Error::Invalid(_) => "invalid",
            anyq_queue_api::Error::NotFound(_) => "not-found",
            anyq_queue_api::Error::Conflict(_) => "conflict",
            anyq_queue_api::Error::Storage(_) => "storage",
            anyq_queue_api::Error::ManagedWorkPending => "managed-work-pending",
            anyq_queue_api::Error::Interrupted => "interrupted",
        };

        Self::Error {
            category: category.into(),
            message: error.to_string(),
        }
    }
}
