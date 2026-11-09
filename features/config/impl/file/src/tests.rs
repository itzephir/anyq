use crate::FileConfig;
use anyq_config_api::{Config, ConfigKey, ConfigValue, Error};
use anyq_config_dispatch::ports::Store;
use std::{
    fs,
    sync::{
        Arc, Barrier,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};

#[test]
fn missing_file_and_round_trip_preserve_typed_values_and_literal_path() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("home/.anyq/config.toml");
    let mut store = FileConfig::new(&path);

    assert_eq!(store.read().unwrap(), Config::default());
    assert!(!path.exists());

    store
        .change(|config| {
            config.set(ConfigKey::TaskAutocreate, ConfigValue::Boolean(true))?;
            config.set(
                ConfigKey::DbPath,
                ConfigValue::Text("a directory/🔥 \\ \".db".into()),
            )
        })
        .unwrap();

    let loaded = FileConfig::new(&path).read().unwrap();

    assert!(loaded.task_autocreate());
    assert!(!loaded.topic_autocreate());
    assert_eq!(loaded.db_path(), Some("a directory/🔥 \\ \".db"));
}

#[test]
fn failed_domain_update_and_invalid_toml_do_not_replace_existing_bytes() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("config.toml");
    let mut store = FileConfig::new(&path);
    store
        .change(|config| config.set(ConfigKey::TaskAutocreate, ConfigValue::Boolean(true)))
        .unwrap();
    let before = fs::read(&path).unwrap();

    assert!(
        store
            .change(|config| config.set(ConfigKey::DbPath, ConfigValue::Text(String::new())))
            .is_err()
    );
    assert_eq!(fs::read(&path).unwrap(), before);

    fs::write(&path, "[broken").unwrap();
    assert!(
        store
            .change(|config| {
                config.unset(ConfigKey::TaskAutocreate);
                Ok(())
            })
            .is_err()
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), "[broken");
}

#[test]
fn independent_file_writers_keep_both_updates() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("config.toml");
    let barrier = Arc::new(Barrier::new(2));
    let threads: Vec<_> = [ConfigKey::TaskAutocreate, ConfigKey::TopicAutocreate]
        .into_iter()
        .map(|key| {
            let path = path.clone();
            let barrier = barrier.clone();

            thread::spawn(move || {
                barrier.wait();
                FileConfig::new(&path)
                    .change(|config| config.set(key, ConfigValue::Boolean(true)))
                    .unwrap();
            })
        })
        .collect();

    for thread in threads {
        thread.join().unwrap();
    }

    let config = FileConfig::new(&path).read().unwrap();

    assert!(config.task_autocreate());
    assert!(config.topic_autocreate());
}

#[test]
fn cancellation_before_replace_removes_temporary_file_and_releases_lock() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("config.toml");
    let cancelled = Arc::new(AtomicBool::new(false));
    let mut store = FileConfig::with_cancellation(&path, cancelled.clone());
    store
        .change(|config| config.set(ConfigKey::TaskAutocreate, ConfigValue::Boolean(true)))
        .unwrap();
    let before = fs::read(&path).unwrap();

    let result = store.change(|config| {
        config.set(ConfigKey::TopicAutocreate, ConfigValue::Boolean(true))?;
        cancelled.store(true, Ordering::SeqCst);
        Ok(())
    });

    assert_eq!(result, Err(Error::Interrupted));
    assert_eq!(fs::read(&path).unwrap(), before);
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 2);

    cancelled.store(false, Ordering::SeqCst);
    store
        .change(|config| config.set(ConfigKey::TopicAutocreate, ConfigValue::Boolean(true)))
        .unwrap();

    assert!(store.read().unwrap().topic_autocreate());
}
