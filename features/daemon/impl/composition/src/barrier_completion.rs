use anyq_queue_api::{BeginCompletion, BeginCompletionRequest, CompleteTask, Error, Result, Task};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

pub(crate) struct BarrierCompletion {
    pub begin: Box<dyn BeginCompletion>,
    pub complete: Box<dyn CompleteTask>,
    pub cancelled: Arc<AtomicBool>,
}

impl CompleteTask for BarrierCompletion {
    fn complete(&mut self, name: &str, force: bool) -> Result<Task> {
        match self.complete.complete(name, force) {
            Err(Error::ManagedWorkPending) => {}
            result => return result,
        }

        let deadline = Instant::now() + Duration::from_secs(5);
        while self.begin.begin(BeginCompletionRequest {
            task: name.into(),
            force,
        })? {
            if self.cancelled.load(Ordering::SeqCst) {
                return Err(Error::Interrupted);
            }
            if Instant::now() >= deadline {
                return Err(Error::Conflict("completion is pending: managed work is not confirmed stopped; inspect plugin status".into()));
            }
            thread::sleep(Duration::from_millis(20));
        }

        self.complete.complete(name, force)
    }
}
