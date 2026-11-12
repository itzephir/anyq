use crate::{CapabilityResult, ManagedWorkDto, TimerDto};
use anyq_queue_wire::{EventDto, MetadataDto};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case", deny_unknown_fields)]
pub enum HostFrame {
    Hello {
        version: u32,
        plugin: String,
        max_frame_bytes: usize,
        state: MetadataDto,
    },
    Event {
        delivery_id: String,
        event: EventDto,
    },
    Timer {
        delivery_id: String,
        timer: TimerDto,
    },
    Cancel {
        work: ManagedWorkDto,
    },
    Result {
        id: String,
        result: CapabilityResult,
    },
    Stop,
}
