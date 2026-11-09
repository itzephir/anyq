use anyq_config_api::{Config, Result};

pub trait Store {
    fn read(&mut self) -> Result<Config>;

    fn change<T>(&mut self, operation: impl FnOnce(&mut Config) -> Result<T>) -> Result<T>;
}
