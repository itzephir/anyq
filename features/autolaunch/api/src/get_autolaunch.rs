use anyq_queue_api::Result;

pub trait GetAutolaunch: Send {
    fn get(&mut self) -> Result<bool>;
}
