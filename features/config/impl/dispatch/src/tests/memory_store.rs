use crate::ports::Store;
use anyq_config_api::{Config, Error, Result};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Default)]
pub struct MemoryStore {
    pub config: Rc<RefCell<Config>>,
    pub fail_commit: bool,
}

impl Store for MemoryStore {
    fn read(&mut self) -> Result<Config> {
        Ok(self.config.borrow().clone())
    }

    fn change<T>(&mut self, operation: impl FnOnce(&mut Config) -> Result<T>) -> Result<T> {
        let mut staged = self.config.borrow().clone();
        let result = operation(&mut staged)?;

        if self.fail_commit {
            return Err(Error::Storage("injected commit failure".into()));
        }

        *self.config.borrow_mut() = staged;

        Ok(result)
    }
}
