use anyq_queue_api::{EventSnapshot, ExecutionDirectory, Result};

pub trait EventRecords {
    fn append(&mut self, snapshot: &EventSnapshot) -> Result<()>;

    fn save_directory(&mut self, task: &str, directory: &ExecutionDirectory) -> Result<()>;

    fn directory(&mut self, task: &str) -> Result<Option<ExecutionDirectory>>;
}
