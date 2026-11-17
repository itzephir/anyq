//! Plugin composition receives focused queue API handles; executables never receive storage access.
mod wall_clock;

use anyq_plugin_api::PluginServices;
use anyq_plugin_dispatch::*;
pub use anyq_plugin_file::install_missing_binary;
use anyq_plugin_process::NativeProcessFactory;
use anyq_plugin_runtime::{Host, HostOptions};
use anyq_plugin_sqlite::SqlitePluginStore;
pub use anyq_plugin_sqlite::initialize;
pub use anyq_plugin_sqlite::validate_existing;
use anyq_queue_api::{GetTask, ReadQueueEvents, Result};
use std::{
    path::Path,
    sync::{Arc, atomic::AtomicBool},
};

pub fn catalog(path: &Path, cancelled: Arc<AtomicBool>) -> Result<anyq_plugin_api::PluginCatalog> {
    let store = anyq_plugin_file::FileCatalog::new(path, cancelled)?;

    Ok(anyq_plugin_api::PluginCatalog {
        list: Box::new(ListInstalledPluginsInteractor::new(store.clone())),
        enable: Box::new(EnableInstalledPluginInteractor::new(store.clone())),
        disable: Box::new(DisableInstalledPluginInteractor::new(store)),
    })
}

pub fn open(path: &Path) -> Result<PluginServices> {
    let store = SqlitePluginStore::open(path)?;

    Ok(PluginServices {
        enable: Box::new(EnablePluginInteractor::new(store.clone())),
        list: Box::new(ListPluginsInteractor::new(store.clone())),
        status: Box::new(GetPluginStatusInteractor::new(store.clone())),
        disable: Box::new(DisablePluginInteractor::new(store)),
    })
}

pub fn run(
    path: &Path,
    directory: &Path,
    events: Box<dyn ReadQueueEvents>,
    read: Box<dyn GetTask>,
    work: WorkCapabilities,
    clock: Option<Box<dyn anyq_plugin_runtime::Clock>>,
    shutdown: Arc<AtomicBool>,
) -> Result<()> {
    Host {
        store: SqlitePluginStore::open_background(path, shutdown.clone())?,
        catalog: catalog(directory, shutdown.clone())?.list,
        events,
        read,
        work,
        clock: clock.unwrap_or_else(|| Box::new(wall_clock::WallClock)),
        processes: Box::new(NativeProcessFactory),
        options: HostOptions::default(),
    }
    .run(shutdown)
}

pub fn ensure_catalog(path: &Path, directory: &Path, cancelled: Arc<AtomicBool>) -> Result<()> {
    let mut catalog = catalog(directory, cancelled)?;
    if let Ok(installed) = catalog.list.list() {
        ensure_catalog_registrations(&mut SqlitePluginStore::open(path)?, &installed)?;
    }

    Ok(())
}

pub use anyq_plugin_runtime::WorkCapabilities;
