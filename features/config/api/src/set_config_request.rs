use crate::{ConfigKey, ConfigValue};

pub struct SetConfigRequest {
    pub key: ConfigKey,
    pub value: ConfigValue,
}
