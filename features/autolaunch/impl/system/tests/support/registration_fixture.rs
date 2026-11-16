use anyq_autolaunch_dispatch::WriteRegistration;
use anyq_autolaunch_system::NativeRegistration;
use std::{
    path::Path,
    sync::{Arc, atomic::AtomicBool},
};

pub struct RegistrationFixture {
    pub registration: NativeRegistration,
    pub cancelled: Arc<AtomicBool>,
    pub home: tempfile::TempDir,
}

impl RegistrationFixture {
    pub fn new() -> Self {
        let home = tempfile::tempdir().unwrap();
        let cancelled = Arc::new(AtomicBool::new(false));
        let registration = NativeRegistration::new(
            home.path(),
            None,
            &std::env::current_exe().unwrap(),
            cancelled.clone(),
        )
        .unwrap();

        Self {
            registration,
            cancelled,
            home,
        }
    }

    pub fn with_source(&self, source: &Path) -> NativeRegistration {
        NativeRegistration::new(
            self.home.path(),
            None,
            source,
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap()
    }
}

impl Drop for RegistrationFixture {
    fn drop(&mut self) {
        // The Windows adapter uses a real profile-specific HKCU Run value.
        // Always remove the isolated registration, including after a failed assertion.
        let _ = self.with_source(Path::new("unused")).unregister();
    }
}
