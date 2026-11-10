use anyq_queue_api::StartPreference;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StartMode {
    Default,
    Start,
    Stop,
    Toggle,
}

impl From<StartPreference> for StartMode {
    fn from(value: StartPreference) -> Self {
        match value {
            StartPreference::Default => Self::Default,
            StartPreference::Start => Self::Start,
            StartPreference::Stop => Self::Stop,
            StartPreference::Toggle => Self::Toggle,
        }
    }
}

impl From<StartMode> for StartPreference {
    fn from(value: StartMode) -> Self {
        match value {
            StartMode::Default => Self::Default,
            StartMode::Start => Self::Start,
            StartMode::Stop => Self::Stop,
            StartMode::Toggle => Self::Toggle,
        }
    }
}
