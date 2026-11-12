use crate::{GetTaskMetadataRequest, MetadataValue, Result};

pub trait GetTaskMetadata {
    fn get(&mut self, request: GetTaskMetadataRequest) -> Result<MetadataValue>;
}
