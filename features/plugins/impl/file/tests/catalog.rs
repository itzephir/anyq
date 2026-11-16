use anyq_plugin_api::{DisableInstalledPlugin, EnableInstalledPlugin, ListInstalledPlugins};
use anyq_plugin_dispatch::{
    CatalogStore, DisableInstalledPluginInteractor, EnableInstalledPluginInteractor,
    ListInstalledPluginsInteractor,
};
use anyq_plugin_file::FileCatalog;
use anyq_queue_api::Error;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc, Barrier,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};

fn store(directory: &Path) -> FileCatalog {
    FileCatalog::new(directory, Arc::new(AtomicBool::new(false))).unwrap()
}

fn binary(directory: &Path, name: &str) -> PathBuf {
    fs::create_dir_all(directory).unwrap();
    let path = directory.join(format!("{name}{}", std::env::consts::EXE_SUFFIX));
    fs::write(&path, b"catalog executable fixture").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
    }

    path
}

#[test]
fn discovery_is_disabled_and_enablement_survives_independent_clients() {
    let temp = tempfile::tempdir().unwrap();
    let directory = temp.path().join("plugins");
    let executable = binary(&directory, "observer");
    let discovered = ListInstalledPluginsInteractor::new(store(&directory))
        .list()
        .unwrap();

    assert_eq!(discovered.len(), 1);
    assert!(!discovered[0].enabled);
    assert!(discovered[0].available);
    assert_eq!(Path::new(&discovered[0].executable), executable);

    let enabled = EnableInstalledPluginInteractor::new(store(&directory))
        .enable("observer")
        .unwrap();
    let restored = ListInstalledPluginsInteractor::new(store(&directory))
        .list()
        .unwrap();

    assert!(enabled.enabled);
    assert_eq!(restored.as_slice(), std::slice::from_ref(&enabled));
    assert_eq!(enabled.revision, 1);

    let disabled = DisableInstalledPluginInteractor::new(store(&directory))
        .disable("observer")
        .unwrap();

    assert!(!disabled.enabled);
    assert_eq!(disabled.revision, 2);
    assert!(executable.is_file());
}

#[test]
fn concurrent_updates_preserve_every_enable_and_do_not_reset_other_revisions() {
    let temp = tempfile::tempdir().unwrap();
    let directory = temp.path().join("plugins");
    let barrier = Arc::new(Barrier::new(8));
    for index in 0..8 {
        binary(&directory, &format!("worker-{index}"));
    }

    let workers: Vec<_> = (0..8)
        .map(|index| {
            let directory = directory.clone();
            let barrier = barrier.clone();
            thread::spawn(move || {
                barrier.wait();
                EnableInstalledPluginInteractor::new(store(&directory))
                    .enable(&format!("worker-{index}"))
                    .unwrap();
            })
        })
        .collect();
    for worker in workers {
        worker.join().unwrap();
    }
    EnableInstalledPluginInteractor::new(store(&directory))
        .enable("worker-0")
        .unwrap();

    let plugins = ListInstalledPluginsInteractor::new(store(&directory))
        .list()
        .unwrap();

    assert_eq!(plugins.len(), 8);
    assert!(plugins.iter().all(|plugin| plugin.enabled));
    assert_eq!(plugins[0].revision, 2);
    assert!(plugins[1..].iter().all(|plugin| plugin.revision == 1));
}

#[test]
fn absent_and_traversal_names_cannot_change_the_catalog() {
    let temp = tempfile::tempdir().unwrap();
    let directory = temp.path().join("plugins");
    binary(&directory, "present");
    EnableInstalledPluginInteractor::new(store(&directory))
        .enable("present")
        .unwrap();
    let original = fs::read(directory.join(".catalog.toml")).unwrap();

    for name in ["missing", "..", ".hidden", "../outside", "a/b", "a\\b"] {
        assert!(
            EnableInstalledPluginInteractor::new(store(&directory))
                .enable(name)
                .is_err()
        );
    }

    assert_eq!(fs::read(directory.join(".catalog.toml")).unwrap(), original);
}

#[test]
fn exceeding_the_enabled_limit_preserves_all_previous_enablements() {
    let temp = tempfile::tempdir().unwrap();
    let directory = temp.path().join("plugins");
    for index in 0..17 {
        binary(&directory, &format!("worker-{index}"));
    }
    for index in 0..16 {
        EnableInstalledPluginInteractor::new(store(&directory))
            .enable(&format!("worker-{index}"))
            .unwrap();
    }
    let original = fs::read(directory.join(".catalog.toml")).unwrap();

    assert!(
        EnableInstalledPluginInteractor::new(store(&directory))
            .enable("worker-16")
            .is_err()
    );

    assert_eq!(fs::read(directory.join(".catalog.toml")).unwrap(), original);
    assert_eq!(
        ListInstalledPluginsInteractor::new(store(&directory))
            .list()
            .unwrap()
            .iter()
            .filter(|plugin| plugin.enabled)
            .count(),
        16
    );
}

#[test]
fn deleting_a_binary_keeps_its_explicit_enablement_visible() {
    let temp = tempfile::tempdir().unwrap();
    let directory = temp.path().join("plugins");
    let executable = binary(&directory, "observer");
    EnableInstalledPluginInteractor::new(store(&directory))
        .enable("observer")
        .unwrap();

    fs::remove_file(executable).unwrap();
    let plugins = ListInstalledPluginsInteractor::new(store(&directory))
        .list()
        .unwrap();

    assert_eq!(plugins.len(), 1);
    assert!(plugins[0].enabled);
    assert!(!plugins[0].available);
    assert!(
        EnableInstalledPluginInteractor::new(store(&directory))
            .enable("observer")
            .is_err()
    );
    assert!(
        !DisableInstalledPluginInteractor::new(store(&directory))
            .disable("observer")
            .unwrap()
            .enabled
    );
}

#[test]
fn cancelled_replacement_preserves_the_previous_complete_snapshot() {
    let temp = tempfile::tempdir().unwrap();
    let directory = temp.path().join("plugins");
    binary(&directory, "observer");
    EnableInstalledPluginInteractor::new(store(&directory))
        .enable("observer")
        .unwrap();
    let original = fs::read(directory.join(".catalog.toml")).unwrap();
    let cancelled = Arc::new(AtomicBool::new(false));
    let mut catalog = FileCatalog::new(&directory, cancelled.clone()).unwrap();

    let result = catalog.change(|records| {
        let mut plugin = records.list()?.remove(0);
        plugin.enabled = false;
        plugin.revision += 1;
        records.save(&plugin)?;
        cancelled.store(true, Ordering::SeqCst);

        Ok(())
    });

    assert!(matches!(result, Err(Error::Interrupted)));
    assert_eq!(fs::read(directory.join(".catalog.toml")).unwrap(), original);
    assert!(
        ListInstalledPluginsInteractor::new(store(&directory))
            .list()
            .unwrap()[0]
            .enabled
    );
}

#[test]
fn malformed_and_oversized_storage_is_rejected_without_replacing_it() {
    let temp = tempfile::tempdir().unwrap();
    let directory = temp.path().join("plugins");
    binary(&directory, "observer");
    for contents in [
        String::new(),
        "version = 99\n[plugins]".into(),
        "x".repeat(1_048_577),
    ] {
        fs::write(directory.join(".catalog.toml"), &contents).unwrap();

        assert!(
            ListInstalledPluginsInteractor::new(store(&directory))
                .list()
                .is_err()
        );
        assert!(
            EnableInstalledPluginInteractor::new(store(&directory))
                .enable("observer")
                .is_err()
        );

        assert_eq!(
            fs::read_to_string(directory.join(".catalog.toml")).unwrap(),
            contents
        );
    }
}

#[cfg(unix)]
#[test]
fn unavailable_permissions_are_rejected_and_invocation_symlinks_are_preserved() {
    use std::os::unix::fs::{PermissionsExt, symlink};

    let temp = tempfile::tempdir().unwrap();
    let directory = temp.path().join("plugins");
    let executable = binary(&directory, "source");
    fs::set_permissions(&executable, fs::Permissions::from_mode(0o600)).unwrap();

    assert!(
        ListInstalledPluginsInteractor::new(store(&directory))
            .list()
            .unwrap()
            .is_empty()
    );
    assert!(
        EnableInstalledPluginInteractor::new(store(&directory))
            .enable("source")
            .is_err()
    );

    fs::set_permissions(&executable, fs::Permissions::from_mode(0o700)).unwrap();
    let invocation = directory.join("observer");
    symlink(&executable, &invocation).unwrap();
    let plugin = EnableInstalledPluginInteractor::new(store(&directory))
        .enable("observer")
        .unwrap();

    assert_eq!(Path::new(&plugin.executable), invocation);
    assert_ne!(Path::new(&plugin.executable), executable);
}
