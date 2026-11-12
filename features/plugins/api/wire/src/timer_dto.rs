use anyq_plugin_api::PluginTimer;
use anyq_queue_api::{Error, Result};
use anyq_queue_wire::MetadataDto;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimerDto {
    pub id: String,
    pub deadline: i64,
    pub payload: MetadataDto,
}

impl From<PluginTimer> for TimerDto {
    fn from(timer: PluginTimer) -> Self {
        Self {
            id: timer.id,
            deadline: timer.deadline,
            payload: timer.payload.into(),
        }
    }
}

impl TimerDto {
    pub fn into_timer(self, origin_sequence: i64) -> Result<PluginTimer> {
        if self.deadline < 0 {
            return Err(Error::Invalid(
                "timer deadline must be nonnegative Unix milliseconds".into(),
            ));
        }

        Ok(PluginTimer {
            id: self.id,
            deadline: self.deadline,
            origin_sequence,
            payload: self.payload.try_into()?,
        })
    }
}
