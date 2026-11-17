use crate::SetAutolaunchRequest;
use anyq_queue_api::Result;

pub trait SetAutolaunch: Send {
    fn set(&mut self, request: SetAutolaunchRequest) -> Result<bool>;
}
