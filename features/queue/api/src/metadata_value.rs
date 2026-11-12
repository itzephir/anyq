use crate::{MetadataBudget, MetadataNumber, Result};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetadataValue {
    Null,
    Bool(bool),
    Number(MetadataNumber),
    String(String),
    Array(Vec<MetadataValue>),
    Object(BTreeMap<String, MetadataValue>),
}

impl MetadataValue {
    /// Eight bytes per value, plus UTF-8 keys/strings and decimal-number bytes.
    pub fn validate(&self) -> Result<()> {
        MetadataBudget::default().include(self, 1)
    }
}
