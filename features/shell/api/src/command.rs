use anyq_queue_api::ExecutionDirectory;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct Command {
    pub program: String,
    pub arguments: Vec<String>,
    pub raw_shell: Option<String>,
    pub directory: ExecutionDirectory,
    pub environment: BTreeMap<String, String>,
}
