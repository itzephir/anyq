use crate::Process;
use anyq_queue_api::{Assignment, Result};
pub trait Runner {
    fn start(&mut self, assignment: &Assignment) -> Result<Box<dyn Process>>;
}
