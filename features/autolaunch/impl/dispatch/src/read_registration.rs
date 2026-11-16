use anyq_queue_api::Result;

pub trait ReadRegistration: Send {
    fn registered(&self) -> Result<bool>;
}
