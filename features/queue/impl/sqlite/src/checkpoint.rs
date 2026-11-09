use anyq_queue_api::{Error, Result};
use std::sync::atomic::{AtomicBool, Ordering};

pub(crate) fn checkpoint(name: &str, cancelled: &AtomicBool) -> Result<()> {
    #[cfg(feature = "integration-test-hooks")]
    anyq_integration_test_hooks::checkpoint(name, cancelled).map_err(Error::Storage)?;
    #[cfg(not(feature = "integration-test-hooks"))]
    let _ = name;

    if cancelled.load(Ordering::SeqCst) {
        return Err(Error::Interrupted);
    }

    Ok(())
}
