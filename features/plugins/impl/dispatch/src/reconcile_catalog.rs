use crate::{Records, Store};
use anyq_plugin_api::{InstalledPlugin, PluginRegistration};
use anyq_queue_api::Result;
use std::collections::BTreeSet;

fn insert_missing(records: &mut dyn Records, installed: &[InstalledPlugin]) -> Result<()> {
    for plugin in installed.iter().filter(|plugin| plugin.enabled) {
        if records.registration(&plugin.name)?.is_some() {
            continue;
        }
        let registration = PluginRegistration {
            name: plugin.name.clone(),
            executable: plugin.executable.clone(),
            arguments: Vec::new(),
            enabled: plugin.enabled && plugin.available,
            catalog_revision: plugin.revision,
            running: false,
            cursor: records.watermark()?,
            failures: 0,
            retry_at: 0,
            last_failure: (!plugin.available)
                .then(|| "installed plugin binary is unavailable".into()),
            subscriptions: Vec::new(),
            capabilities: Vec::new(),
            state: Default::default(),
        };
        records.save(&registration)?;
    }

    Ok(())
}

pub fn ensure_catalog_registrations(
    store: &mut impl Store,
    installed: &[InstalledPlugin],
) -> Result<()> {
    store.transaction(|records| insert_missing(records, installed))
}

pub fn reconcile_catalog(
    store: &mut impl Store,
    installed: Option<&[InstalledPlugin]>,
    active: &BTreeSet<String>,
) -> Result<()> {
    store.transaction(|records| {
        if let Some(installed) = installed {
            insert_missing(records, installed)?;
        }
        for mut registration in records.registrations()? {
            let original = registration.clone();
            let desired = installed.and_then(|plugins| {
                plugins
                    .iter()
                    .find(|plugin| plugin.name == registration.name)
            });
            match desired {
                Some(plugin) => {
                    let command_changed = registration.executable != plugin.executable
                        || !registration.arguments.is_empty();
                    if active.contains(&registration.name)
                        && (command_changed || !registration.enabled && plugin.enabled)
                    {
                        registration.enabled = false;
                        registration.last_failure =
                            Some("plugin restart is waiting for managed cancellation".into());
                    } else {
                        let enabled = plugin.enabled && plugin.available;
                        let refreshed = registration.catalog_revision != plugin.revision
                            || command_changed
                            || registration.enabled != enabled;
                        registration.executable = plugin.executable.clone();
                        registration.arguments.clear();
                        registration.enabled = enabled;
                        registration.catalog_revision = plugin.revision;
                        if refreshed {
                            registration.failures = 0;
                            registration.retry_at = 0;
                            registration.last_failure = None;
                        }
                        if plugin.enabled && !plugin.available {
                            registration.last_failure =
                                Some("installed plugin binary is unavailable".into());
                        }
                    }
                }
                None => {
                    registration.enabled = false;
                    if installed.is_none() {
                        registration.last_failure =
                            Some("user plugin catalog is unavailable".into());
                    }
                }
            }
            if registration != original {
                records.save(&registration)?;
            }
        }

        Ok(())
    })
}
