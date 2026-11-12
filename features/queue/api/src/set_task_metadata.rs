use crate::{Metadata, Result, SetTaskMetadataRequest};

pub trait SetTaskMetadata {
    fn set(&mut self, request: SetTaskMetadataRequest) -> Result<Metadata>;
}
