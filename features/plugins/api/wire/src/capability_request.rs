use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "kebab-case", deny_unknown_fields)]
pub enum CapabilityRequest {
    ReadTask {
        name: String,
    },
    Complete {
        action_id: String,
        task: String,
        #[serde(default)]
        force: bool,
    },
    ReadState,
    ManagedRegister {
        details: anyq_queue_wire::MetadataDto,
        action_id: String,
        task: String,
    },
    ManagedReport {
        details: Option<anyq_queue_wire::MetadataDto>,
        action_id: String,
        state: String,
    },
    ManagedList,
}

impl CapabilityRequest {
    pub fn capability(&self) -> &'static str {
        match self {
            Self::ReadTask { .. } => "read",
            Self::Complete { .. } => "complete",
            Self::ReadState => "state",
            Self::ManagedRegister { .. } | Self::ManagedReport { .. } | Self::ManagedList => {
                "managed"
            }
        }
    }
}
