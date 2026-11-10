use crate::error::{Error, Result};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

pub fn install() -> Result<Arc<AtomicBool>> {
    let cancelled = Arc::new(AtomicBool::new(false));

    // Detached Windows owners have no console. Their lifecycle uses the local
    // control protocol; foreground owners retain native console signals.
    #[cfg(windows)]
    if std::env::var_os("ANYQ_DAEMON_BACKGROUND").is_some() {
        return Ok(cancelled);
    }

    let signal = cancelled.clone();
    ctrlc::set_handler(move || {
        if signal.swap(true, Ordering::SeqCst) {
            std::process::exit(130);
        }
    })
    .map_err(|error| Error::Invalid(format!("cannot handle interrupts: {error}")))?;

    Ok(cancelled)
}
