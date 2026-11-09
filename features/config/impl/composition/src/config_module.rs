use anyq_config_api::{GetConfig, GetConfigValue, SetConfigValue, UnsetConfigValue};

pub struct ConfigModule {
    pub get_config: Box<dyn GetConfig>,
    pub get_value: Box<dyn GetConfigValue>,
    pub set_value: Box<dyn SetConfigValue>,
    pub unset_value: Box<dyn UnsetConfigValue>,
}
