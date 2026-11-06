use super::UnitOfWork;
use anyq_queue_api::Result;

pub trait Store {
    /// Every write MUST roll back when the operation returns an error.
    fn transaction<T>(
        &mut self,
        operation: impl FnOnce(&mut dyn UnitOfWork) -> Result<T>,
    ) -> Result<T>;
}
