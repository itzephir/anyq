use crate::*;

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
