use crate::{ManagedWork, Result};

pub trait CancelManagedWork {
    fn cancel(&mut self, owner: &str, action_id: &str) -> Result<ManagedWork>;
}
