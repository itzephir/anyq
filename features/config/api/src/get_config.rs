use crate::{Config, Result};

pub trait GetConfig {
    fn get(&mut self) -> Result<Config>;
}
