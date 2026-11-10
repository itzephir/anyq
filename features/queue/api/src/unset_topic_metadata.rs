use crate::{Metadata, Result, UnsetTopicMetadataRequest};

pub trait UnsetTopicMetadata {
    fn unset(&mut self, request: UnsetTopicMetadataRequest) -> Result<Metadata>;
}
