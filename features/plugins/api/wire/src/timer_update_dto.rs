use crate::TimerDto;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "kebab-case", deny_unknown_fields)]
pub enum TimerUpdateDto {
    Put { timer: TimerDto },
    Remove { id: String },
}
