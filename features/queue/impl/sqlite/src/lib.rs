//! Transactional SQLite adapters; one responsibility per adapter.
mod sqlite;
mod sqlite_deliveries;
mod sqlite_groups;
mod sqlite_messages;
mod sqlite_topics;
mod sqlite_unit_of_work;
mod storage_error;
pub use sqlite::Sqlite;
#[cfg(test)]
mod tests;
