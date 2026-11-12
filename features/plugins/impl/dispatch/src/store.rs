use crate::Records;
use anyq_queue_api::Result;

pub trait Store {
    fn transaction<T>(
        &mut self,
        operation: impl FnOnce(&mut dyn Records) -> Result<T>,
    ) -> Result<T>;
}
