use anyq_queue_api::{Queue, Result};
use anyq_queue_dispatch::{
    Dispatch,
    ports::{Clock, Tokens},
};
use anyq_queue_sqlite::Sqlite;
use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

struct SystemClock;
impl Clock for SystemClock {
    fn now_ms(&self) -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis().min(i64::MAX as u128) as i64)
            .unwrap_or(0)
    }
}
struct UuidTokens;
impl Tokens for UuidTokens {
    fn next(&mut self) -> String {
        uuid::Uuid::new_v4().to_string()
    }
}
pub fn open(path: &Path) -> Result<Box<dyn Queue>> {
    Ok(Box::new(Dispatch::new(
        Sqlite::open(path)?,
        SystemClock,
        UuidTokens,
    )))
}
