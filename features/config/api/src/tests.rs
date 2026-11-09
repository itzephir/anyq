use crate::*;

#[test]
fn only_containers_of_known_keys_are_valid_configuration_sections() {
    for section in ["task", "topic", "db"] {
        assert!(ConfigKey::require_section(section).is_ok());
    }

    for section in [
        "",
        "list",
        "unknown",
        "task.unknown",
        "task.autocreate",
        "topic.autocreate",
        "db.path",
        "task.",
        "Task",
        "task2",
    ] {
        assert!(ConfigKey::require_section(section).is_err(), "{section}");
    }
}

#[test]
fn defaults_overrides_and_unset_have_distinct_persistence() {
    let mut config = Config::default();

    assert!(!config.task_autocreate());
    assert!(!config.topic_autocreate());
    assert_eq!(config.db_path(), None);
    assert!(config.overrides().is_empty());

    config
        .set(ConfigKey::TaskAutocreate, ConfigValue::Boolean(true))
        .unwrap();
    config
        .set(ConfigKey::TopicAutocreate, ConfigValue::Boolean(false))
        .unwrap();
    config
        .set(
            ConfigKey::DbPath,
            ConfigValue::Text("a directory/🔥.db".into()),
        )
        .unwrap();

    assert!(config.task_autocreate());
    assert_eq!(config.overrides().len(), 3);
    assert_eq!(config.db_path(), Some("a directory/🔥.db"));

    config.unset(ConfigKey::TaskAutocreate);
    config.unset(ConfigKey::DbPath);

    assert!(!config.task_autocreate());
    assert_eq!(config.db_path(), None);
    assert_eq!(
        config.overrides(),
        [(ConfigKey::TopicAutocreate, ConfigValue::Boolean(false))]
    );
}

#[test]
fn invalid_keys_types_and_paths_do_not_mutate_configuration() {
    let mut config = Config::default();
    config
        .set(ConfigKey::TaskAutocreate, ConfigValue::Boolean(true))
        .unwrap();
    let before = config.clone();

    for (key, value) in [
        (ConfigKey::TaskAutocreate, ConfigValue::Text("false".into())),
        (ConfigKey::DbPath, ConfigValue::Boolean(true)),
        (ConfigKey::DbPath, ConfigValue::Text(String::new())),
    ] {
        assert!(config.set(key, value).is_err());
        assert_eq!(config, before);
    }

    for value in ["yes", "1", "TRUE", "", "false "] {
        assert!(ConfigValue::parse(ConfigKey::TaskAutocreate, value).is_err());
    }

    assert!(ConfigKey::parse("task.unknown").is_err());
    assert!(ConfigKey::parse("list").is_err());
    assert_eq!(
        ConfigValue::parse(ConfigKey::TopicAutocreate, "false").unwrap(),
        ConfigValue::Boolean(false)
    );
}

#[test]
fn task_autostart_defaults_to_true_and_persists_explicit_overrides() {
    let mut config = Config::default();

    assert!(config.task_autostart());
    assert_eq!(
        config.get(ConfigKey::TaskAutostart),
        Some(ConfigValue::Boolean(true))
    );
    assert!(
        !config
            .overrides()
            .iter()
            .any(|(key, _)| *key == ConfigKey::TaskAutostart)
    );

    config
        .set(ConfigKey::TaskAutostart, ConfigValue::Boolean(false))
        .unwrap();

    assert!(!config.task_autostart());
    assert_eq!(
        config.overrides(),
        [(ConfigKey::TaskAutostart, ConfigValue::Boolean(false))]
    );
    assert!(
        config
            .set(ConfigKey::TaskAutostart, ConfigValue::Text("false".into()))
            .is_err()
    );
    assert!(!config.task_autostart());
    config.unset(ConfigKey::TaskAutostart);
    assert!(config.task_autostart());
    assert!(config.overrides().is_empty());
}
