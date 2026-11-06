use anyq_queue_api::Result;

pub trait Process {
    /// None means running; Some contains the exit code (signals are nonzero).
    fn poll(&mut self) -> Result<Option<i32>>;

    fn stop(&mut self) -> Result<()>;
}
