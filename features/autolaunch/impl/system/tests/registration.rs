#[path = "support/registration_fixture.rs"]
mod registration_fixture;

use anyq_autolaunch_dispatch::{ReadRegistration, WriteRegistration};
use anyq_queue_api::Error;
use registration_fixture::RegistrationFixture;
use std::{fs, sync::atomic::Ordering};

#[test]
fn native_registration_is_persistent_and_removal_preserves_retained_launcher() {
    let fixture = RegistrationFixture::new();
    assert!(!fixture.registration.registered().unwrap());

    fixture.registration.register().unwrap();
    let command = fixture.registration.registered_command().unwrap().unwrap();
    let independent = fixture.with_source(&fixture.home.path().join("absent-installation"));
    assert!(independent.registered().unwrap());
    assert_eq!(&command[1..], &["daemon", "start"]);
    assert_eq!(
        fs::read(&command[0]).unwrap(),
        fs::read(std::env::current_exe().unwrap()).unwrap()
    );

    independent.unregister().unwrap();
    independent.unregister().unwrap();

    assert!(!fixture.registration.registered().unwrap());
    assert!(std::path::Path::new(&command[0]).is_file());
}

#[test]
fn failed_or_cancelled_update_preserves_previous_native_registration() {
    let fixture = RegistrationFixture::new();
    fixture.registration.register().unwrap();
    let original = fixture.registration.registered_command().unwrap();

    let missing = fixture.with_source(&fixture.home.path().join("missing.exe"));
    assert!(missing.register().is_err());
    fixture.cancelled.store(true, Ordering::SeqCst);
    assert!(matches!(
        fixture.registration.unregister(),
        Err(Error::Interrupted)
    ));
    fixture.cancelled.store(false, Ordering::SeqCst);

    assert_eq!(fixture.registration.registered_command().unwrap(), original);
    assert_eq!(missing.registered_command().unwrap(), original);
}

#[test]
fn unavailable_retained_image_is_reported_and_off_can_repair_registration() {
    let fixture = RegistrationFixture::new();
    fixture.registration.register().unwrap();
    let command = fixture.registration.registered_command().unwrap().unwrap();
    fs::remove_file(&command[0]).unwrap();

    assert!(fixture.registration.registered().is_err());
    fixture.registration.unregister().unwrap();

    assert!(!fixture.registration.registered().unwrap());
}

#[cfg(target_os = "linux")]
#[test]
fn empty_or_relative_xdg_configuration_uses_the_native_default_directory() {
    use anyq_autolaunch_system::NativeRegistration;
    use std::{path::Path, sync::Arc, sync::atomic::AtomicBool};

    let fixture = RegistrationFixture::new();

    for configured in ["", "relative/config"] {
        let registration = NativeRegistration::new(
            fixture.home.path(),
            Some(Path::new(configured)),
            &std::env::current_exe().unwrap(),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
        assert!(!registration.registered().unwrap());

        registration.register().unwrap();

        assert!(
            fixture
                .home
                .path()
                .join(".config/autostart/anyq.desktop")
                .is_file()
        );
        assert!(fixture.registration.registered().unwrap());

        registration.unregister().unwrap();

        assert!(!fixture.registration.registered().unwrap());
    }
}
