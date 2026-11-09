use crate::{Config, Result, SetConfigRequest};

pub trait SetConfigValue {
    fn set(&mut self, request: SetConfigRequest) -> Result<Config>;
}
