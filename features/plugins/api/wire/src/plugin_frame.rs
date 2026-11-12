use crate::{CapabilityRequest, TimerUpdateDto};
use anyq_queue_wire::MetadataDto;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case", deny_unknown_fields)]
pub enum PluginFrame {
    Hello {
        version: u32,
        subscriptions: Vec<String>,
        capabilities: Vec<String>,
    },
    Ack {
        delivery_id: String,
        state: Option<MetadataDto>,
        #[serde(default)]
        timers: Vec<TimerUpdateDto>,
    },
    Request {
        id: String,
        request: CapabilityRequest,
    },
}
