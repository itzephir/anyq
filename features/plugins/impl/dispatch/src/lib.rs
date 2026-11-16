//! Explicit local plugin boundary.

mod records;
pub use records::Records;

mod store;
pub use store::Store;

mod enable_plugin;
pub use enable_plugin::EnablePluginInteractor;

mod list_plugins;
pub use list_plugins::ListPluginsInteractor;

mod get_plugin_status;
pub use get_plugin_status::GetPluginStatusInteractor;

mod disable_plugin;
pub use disable_plugin::DisablePluginInteractor;

mod acknowledge;
pub use acknowledge::acknowledge;

mod read_status;

mod catalog_store;
pub use catalog_store::CatalogStore;

mod catalog_records;
pub use catalog_records::CatalogRecords;

mod list_installed_plugins;
pub use list_installed_plugins::ListInstalledPluginsInteractor;

mod enable_installed_plugin;
pub use enable_installed_plugin::EnableInstalledPluginInteractor;

mod disable_installed_plugin;
pub use disable_installed_plugin::DisableInstalledPluginInteractor;

mod reconcile_catalog;
pub use reconcile_catalog::{ensure_catalog_registrations, reconcile_catalog};
