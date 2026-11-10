use crate::{ManagedWork, ResolveManagedWorkRequest, Result};

pub trait ResolveManagedWork {
    fn resolve(&mut self, request: ResolveManagedWorkRequest) -> Result<ManagedWork>;
}
