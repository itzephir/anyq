use anyq_queue_api::{Error, Metadata, Result};

pub(crate) fn restore_metadata(raw: &str) -> Result<Metadata> {
    anyq_queue_metadata::parse_json(raw)
        .and_then(Metadata::new)
        .map_err(|error| Error::Storage(format!("invalid stored metadata: {error}")))
}
