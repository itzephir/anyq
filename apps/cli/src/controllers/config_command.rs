use anyq_config_api::{ConfigKey, SetConfigRequest};

pub enum ConfigCommand {
    List,
    Get(ConfigKey),
    Set(SetConfigRequest),
    Unset(ConfigKey),
}
