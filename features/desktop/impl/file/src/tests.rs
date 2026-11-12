use crate::FilePreferences;
use anyq_desktop_api::*;
use anyq_desktop_dispatch::ports::PreferenceStore;

#[test]
fn save_restart_reset_preserves_shared_configuration() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("panel.toml");
    let shared = dir.path().join("config.toml");
    std::fs::write(&shared, "db.path = 'cli.db'\n").unwrap();
    let mut store = FilePreferences::new(&path);
    assert_eq!(store.load().unwrap(), PanelPreferences::default());
    assert!(!path.exists());
    let preferences = PanelPreferences {
        database_override: Some("C:\\queues\\тест.db".into()),
        theme: Theme::Dark,
        language: Language::Russian,
    };

    store.save(&preferences).unwrap();
    let mut restarted = FilePreferences::new(&path);

    assert_eq!(restarted.load().unwrap(), preferences);
    restarted.save(&PanelPreferences::default()).unwrap();
    assert_eq!(store.load().unwrap(), PanelPreferences::default());
    assert_eq!(
        std::fs::read_to_string(shared).unwrap(),
        "db.path = 'cli.db'\n"
    );
}

#[test]
fn invalid_or_failed_writes_preserve_the_previous_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("panel.toml");
    let mut store = FilePreferences::new(&path);
    store.save(&PanelPreferences::default()).unwrap();
    let before = std::fs::read(&path).unwrap();
    let invalid = PanelPreferences {
        database_override: Some(String::new()),
        ..Default::default()
    };

    assert!(store.save(&invalid).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), before);

    std::fs::remove_file(path.with_extension("toml.lock")).unwrap();
    std::fs::create_dir(path.with_extension("toml.lock")).unwrap();

    assert!(store.save(&PanelPreferences::default()).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), before);
}

#[test]
fn unknown_or_malformed_preferences_are_not_rewritten() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("panel.toml");
    let mut store = FilePreferences::new(&path);

    for text in [
        "future = true",
        "language = 7",
        "theme = 'other'",
        "database_override = ''",
    ] {
        std::fs::write(&path, text).unwrap();

        assert!(store.load().is_err());
        assert!(store.save(&PanelPreferences::default()).is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), text);
    }
}
