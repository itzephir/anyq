use crate::{Metadata, ReplaceTopicMetadataRequest, Result};

pub trait ReplaceTopicMetadata {
    fn replace(&mut self, request: ReplaceTopicMetadataRequest) -> Result<Metadata>;
}
