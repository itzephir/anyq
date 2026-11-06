use super::memory::Memory;
use crate::ports::{Store, UnitOfWork};
use anyq_queue_api::Result;
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Default)]
pub struct MemoryStore {
    pub data: Rc<RefCell<Memory>>,
}

impl Store for MemoryStore {
    fn transaction<T>(
        &mut self,
        operation: impl FnOnce(&mut dyn UnitOfWork) -> Result<T>,
    ) -> Result<T> {
        let mut staged = self.data.borrow().clone();
        let result = operation(&mut staged)?;
        *self.data.borrow_mut() = staged;

        Ok(result)
    }
}
