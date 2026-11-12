//! Explicit local plugin boundary.

mod initialize;
pub use initialize::initialize;

mod sqlite_plugin_store;
pub use sqlite_plugin_store::SqlitePluginStore;

mod sqlite_records;
mod storage;

#[cfg(test)]
mod tests;

mod validate_existing;
pub use validate_existing::validate_existing;
