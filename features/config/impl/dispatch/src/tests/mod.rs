mod memory_store;

use crate::*;
use anyq_config_api::*;
use memory_store::MemoryStore;

#[test]
fn separate_config_contracts_read_set_and_unset_typed_values() {
    let store = MemoryStore::default();
    SetConfigValueInteractor::new(store.clone())
        .set(SetConfigRequest {
            key: ConfigKey::TaskAutocreate,
            value: ConfigValue::Boolean(true),
        })
        .unwrap();

    assert!(
        GetConfigInteractor::new(store.clone())
            .get()
            .unwrap()
            .task_autocreate()
    );
    assert_eq!(
        GetConfigValueInteractor::new(store.clone())
            .get(ConfigKey::TaskAutocreate)
            .unwrap(),
        Some(ConfigValue::Boolean(true))
    );

    UnsetConfigValueInteractor::new(store.clone())
        .unset(ConfigKey::TaskAutocreate)
        .unwrap();

    assert!(
        !GetConfigInteractor::new(store.clone())
            .get()
            .unwrap()
            .task_autocreate()
    );
    assert!(store.config.borrow().overrides().is_empty());
}

#[test]
fn invalid_updates_and_failed_commits_preserve_the_previous_aggregate() {
    let mut store = MemoryStore::default();
    let request = || SetConfigRequest {
        key: ConfigKey::TaskAutocreate,
        value: ConfigValue::Boolean(true),
    };
    store.fail_commit = true;

    assert!(
        SetConfigValueInteractor::new(store.clone())
            .set(request())
            .is_err()
    );
    assert_eq!(*store.config.borrow(), Config::default());

    store.fail_commit = false;
    assert!(
        SetConfigValueInteractor::new(store.clone())
            .set(SetConfigRequest {
                key: ConfigKey::DbPath,
                value: ConfigValue::Text(String::new()),
            })
            .is_err()
    );
    assert_eq!(*store.config.borrow(), Config::default());
}
