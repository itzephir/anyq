use crate::Run;
use anyq_queue_api::Result;

pub trait Worker {
    fn run_one(&mut self) -> Result<Run>;

    fn idle(&mut self);

    fn stopped(&self) -> bool;
}
