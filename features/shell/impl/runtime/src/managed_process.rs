use anyq_queue_api::Result;
use anyq_shell_api::{EntryOutcome, EntryOutput};

pub trait ManagedProcess {
    /// Some is proof that the whole managed tree stopped and both pipes reached EOF.
    fn poll(&mut self) -> Result<Option<EntryOutcome>>;
    fn cancel(&mut self) -> Result<()>;
    fn output(&self) -> EntryOutput;
}
