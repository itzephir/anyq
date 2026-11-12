use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GateDto {
    pub name: Option<String>,
    pub hit: usize,
    pub directory: Option<PathBuf>,
}
