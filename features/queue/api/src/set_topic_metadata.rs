use crate::{Metadata, Result, SetTopicMetadataRequest};

pub trait SetTopicMetadata {
    fn set(&mut self, request: SetTopicMetadataRequest) -> Result<Metadata>;
}
