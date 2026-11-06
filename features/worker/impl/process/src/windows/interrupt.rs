use std::{
    io,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
pub struct Interrupt;
impl Interrupt {
    pub fn install(cancelled: Arc<AtomicBool>) -> io::Result<Self> {
        ctrlc::set_handler(move || {
            cancelled.store(true, Ordering::Relaxed);
        })
        .map_err(io::Error::other)?;
        Ok(Self)
    }
}
