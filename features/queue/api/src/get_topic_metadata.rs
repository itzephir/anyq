use crate::{GetTopicMetadataRequest, MetadataValue, Result};

pub trait GetTopicMetadata {
    fn get(&mut self, request: GetTopicMetadataRequest) -> Result<MetadataValue>;
}
