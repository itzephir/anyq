use crate::DatabaseSource;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseSelection {
    pub path: PathBuf,
    pub source: DatabaseSource,
}
