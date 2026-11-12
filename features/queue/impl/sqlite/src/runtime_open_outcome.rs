use crate::Sqlite;

pub enum RuntimeOpen {
    /// Complete current queue and injected feature schemas; no migration ran.
    Ready(Sqlite),
    Missing,
    Uninitialized,
    /// A normal queue request must perform the supported upgrade first.
    UpgradeRequired(i64),
    Incompatible(String),
}
