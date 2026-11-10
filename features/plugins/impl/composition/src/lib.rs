//! Plugin composition receives focused queue API handles; executables never receive storage access.
mod wall_clock;

use anyq_plugin_api::PluginServices;
use anyq_plugin_dispatch::*;
use anyq_plugin_process::NativeProcessFactory;
use anyq_plugin_runtime::{Host, HostOptions};
use anyq_plugin_sqlite::SqlitePluginStore;
pub use anyq_plugin_sqlite::initialize;
use anyq_queue_api::{GetTask, ReadQueueEvents, Result};
use std::{
    path::Path,
    sync::{Arc, atomic::AtomicBool},
};

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
    events: Box<dyn ReadQueueEvents>,
    read: Box<dyn GetTask>,
    work: WorkCapabilities,
    clock: Option<Box<dyn anyq_plugin_runtime::Clock>>,
    shutdown: Arc<AtomicBool>,
) -> Result<()> {
    Host {
        store: SqlitePluginStore::open(path)?,
        events,
        read,
        work,
        clock: clock.unwrap_or_else(|| Box::new(wall_clock::WallClock)),
        processes: Box::new(NativeProcessFactory),
        options: HostOptions::default(),
    }
    .run(shutdown)
}

pub use anyq_plugin_runtime::WorkCapabilities;
