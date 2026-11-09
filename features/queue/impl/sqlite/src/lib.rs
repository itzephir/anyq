//! SQLite adapter for the named-task transaction boundary.
mod begin_transaction;
mod checkpoint;
mod sqlite;
pub use sqlite::Sqlite;

mod sqlite_tasks;
mod sqlite_topics;
mod sqlite_unit_of_work;
mod storage_error;

#[cfg(test)]
mod tests;
