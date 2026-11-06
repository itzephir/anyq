use crate::{io, platform};
use anyq_queue_api::Result;
use anyq_worker_execution::Process;

impl Process for ManagedProcess {
    fn poll(&mut self) -> Result<Option<i32>> {
        self.tree.poll().map_err(io)
    }

    fn stop(&mut self) -> Result<()> {
        self.tree.stop().map_err(io)
    }
}

pub(crate) struct ManagedProcess {
    pub tree: platform::ChildTree,
}
