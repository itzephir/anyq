use crate::{ManagedLaunch, RegisterManagedWorkRequest, Result};

pub trait RegisterManagedWork {
    fn register(&mut self, request: RegisterManagedWorkRequest) -> Result<ManagedLaunch>;
}
