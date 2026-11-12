use crate::{Error, Metadata, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataPath(Vec<String>);

impl MetadataPath {
    pub fn parse(path: &str) -> Result<Self> {
        if path.len() > Metadata::MAX_BYTES {
            return Err(Error::Invalid("metadata path exceeds 1 MiB".into()));
        }
        let parts: Vec<_> = path.split('.').map(str::to_owned).collect();
        if parts.iter().any(String::is_empty) || parts.len() >= Metadata::MAX_DEPTH {
            return Err(Error::Invalid("invalid metadata dotted path".into()));
        }

        Ok(Self(parts))
    }

    pub fn segments(&self) -> &[String] {
        &self.0
    }
}
