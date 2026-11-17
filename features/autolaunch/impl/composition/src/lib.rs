use anyq_autolaunch_api::AutolaunchServices;
use anyq_autolaunch_dispatch::{GetAutolaunchInteractor, SetAutolaunchInteractor};
use anyq_autolaunch_system::NativeRegistration;
use anyq_queue_api::Result;
use std::{
    path::Path,
    sync::{Arc, atomic::AtomicBool},
};

pub use anyq_autolaunch_system::retain_launcher;

pub fn open(
    home: &Path,
    configuration_directory: Option<&Path>,
    executable: &Path,
    cancelled: Arc<AtomicBool>,
) -> Result<AutolaunchServices> {
    let registration =
        NativeRegistration::new(home, configuration_directory, executable, cancelled)?;

    Ok(AutolaunchServices {
        get: Box::new(GetAutolaunchInteractor::new(registration.clone())),
        set: Box::new(SetAutolaunchInteractor::new(registration)),
    })
}
