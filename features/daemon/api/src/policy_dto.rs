use anyq_queue_api::{AutoCreatePolicy, QueuePolicy};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyDto {
    pub tasks: bool,
    pub topics: bool,
    pub autostart: bool,
}

impl From<QueuePolicy> for PolicyDto {
    fn from(value: QueuePolicy) -> Self {
        Self {
            tasks: value.autocreate.tasks,
            topics: value.autocreate.topics,
            autostart: value.task_autostart,
        }
    }
}

impl From<PolicyDto> for QueuePolicy {
    fn from(value: PolicyDto) -> Self {
        Self {
            autocreate: AutoCreatePolicy {
                tasks: value.tasks,
                topics: value.topics,
            },
            task_autostart: value.autostart,
        }
    }
}
