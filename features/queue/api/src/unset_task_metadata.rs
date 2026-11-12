use crate::{Metadata, Result, UnsetTaskMetadataRequest};

pub trait UnsetTaskMetadata {
    fn unset(&mut self, request: UnsetTaskMetadataRequest) -> Result<Metadata>;
}
