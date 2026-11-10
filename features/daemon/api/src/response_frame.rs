use crate::QueueResponse;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseFrame {
    pub version: u32,
    pub id: u64,
    pub response: QueueResponse,
}
