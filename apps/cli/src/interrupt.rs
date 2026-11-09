use crate::error::{Error, Result};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

pub fn install() -> Result<Arc<AtomicBool>> {
    let cancelled = Arc::new(AtomicBool::new(false));
    let signal = cancelled.clone();
    ctrlc::set_handler(move || {
        if signal.swap(true, Ordering::SeqCst) {
            std::process::exit(130);
        }
    })
    .map_err(|error| Error::Invalid(format!("cannot handle interrupts: {error}")))?;

    Ok(cancelled)
}
