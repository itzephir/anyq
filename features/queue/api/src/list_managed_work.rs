use crate::{ListManagedWorkRequest, ManagedWork, Result};

pub trait ListManagedWork {
    fn list(&mut self, request: ListManagedWorkRequest) -> Result<Vec<ManagedWork>>;
}
