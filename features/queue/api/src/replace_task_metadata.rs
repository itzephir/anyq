use crate::{Metadata, ReplaceTaskMetadataRequest, Result};

pub trait ReplaceTaskMetadata {
    fn replace(&mut self, request: ReplaceTaskMetadataRequest) -> Result<Metadata>;
}
