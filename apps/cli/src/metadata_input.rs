use crate::error::{Error, Result};
use anyq_queue_api::{Metadata, MetadataValue};
use std::{fs::File, io::Read, path::Path};

/// Format functions are injected by main; local file IO remains a CLI adapter.
pub struct MetadataInput {
    pub parse_value: fn(&str) -> anyq_queue_api::Result<MetadataValue>,
    pub parse_json: fn(&str) -> anyq_queue_api::Result<MetadataValue>,
    pub parse_yaml: fn(&str) -> anyq_queue_api::Result<Metadata>,
    pub parse_toml: fn(&str) -> anyq_queue_api::Result<Metadata>,
}

impl MetadataInput {
    pub fn read(&self, path: &Path) -> Result<Metadata> {
        let extension = path
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if !matches!(extension.as_str(), "json" | "yaml" | "yml" | "toml") {
            return Err(Error::Invalid(
                "metadata file must use .json, .yaml, .yml or .toml".into(),
            ));
        }
        let mut bytes = Vec::new();
        File::open(path)?
            .take(Metadata::MAX_BYTES as u64 + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() > Metadata::MAX_BYTES {
            return Err(Error::Invalid("metadata file exceeds 1 MiB".into()));
        }
        let text = String::from_utf8(bytes)
            .map_err(|_| Error::Invalid("metadata file must be UTF-8".into()))?;

        Ok(match extension.as_str() {
            "json" => Metadata::new((self.parse_json)(&text)?)?,
            "yaml" | "yml" => (self.parse_yaml)(&text)?,
            "toml" => (self.parse_toml)(&text)?,
            _ => unreachable!("validated extension"),
        })
    }

    pub fn initial(&self, file: Option<&Path>, patches: &[String]) -> Result<Metadata> {
        if patches.len() % 2 != 0 {
            return Err(Error::Invalid(
                "metadata patches require KEY VALUE pairs".into(),
            ));
        }

        let mut metadata = file
            .map(|path| self.read(path))
            .transpose()?
            .unwrap_or_default();

        for pair in patches.chunks_exact(2) {
            metadata.set(&pair[0], (self.parse_value)(&pair[1])?)?;
        }

        Ok(metadata)
    }
}
