pub use anyq_config_file::paths;
mod config_module;
pub use config_module::ConfigModule;

use anyq_config_dispatch::{
    GetConfigInteractor, GetConfigValueInteractor, SetConfigValueInteractor,
    UnsetConfigValueInteractor,
};
use anyq_config_file::FileConfig;
use std::{
    path::Path,
    sync::{Arc, atomic::AtomicBool},
};

pub fn open(path: &Path, cancelled: Arc<AtomicBool>) -> ConfigModule {
    let store = FileConfig::with_cancellation(path, cancelled);

    ConfigModule {
        get_config: Box::new(GetConfigInteractor::new(store.clone())),
        get_value: Box::new(GetConfigValueInteractor::new(store.clone())),
        set_value: Box::new(SetConfigValueInteractor::new(store.clone())),
        unset_value: Box::new(UnsetConfigValueInteractor::new(store)),
    }
}
