//! SQLite adapter for the named-task transaction boundary.
mod begin_transaction;
mod checkpoint;
mod sqlite;
pub use sqlite::Sqlite;

mod runtime_open;
mod runtime_open_outcome;
mod validate_runtime;
pub use runtime_open_outcome::RuntimeOpen;

mod restore_metadata;

mod observation;
mod sqlite_tasks;
mod sqlite_topics;
mod sqlite_unit_of_work;
mod storage_error;

#[cfg(test)]
mod tests;

mod read_events;
mod sqlite_events;
mod system_clock;

mod retry_lock;
mod sqlite_work;
