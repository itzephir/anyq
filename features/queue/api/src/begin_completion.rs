use crate::{BeginCompletionRequest, Result};

pub trait BeginCompletion {
    fn begin(&mut self, request: BeginCompletionRequest) -> Result<bool>;
}
