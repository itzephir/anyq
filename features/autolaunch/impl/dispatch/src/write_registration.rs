use anyq_queue_api::Result;

pub trait WriteRegistration: Send {
    fn register(&self) -> Result<()>;
    fn unregister(&self) -> Result<()>;
}
