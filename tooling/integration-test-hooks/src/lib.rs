mod gate_context;
pub use gate_context::GateContext;

use std::{
    cell::RefCell,
    env, fs,
    path::PathBuf,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    thread,
    time::Duration,
};

static HITS: AtomicUsize = AtomicUsize::new(0);

thread_local! {
    static CONTEXT: RefCell<Option<GateContext>> = const { RefCell::new(None) };
}

/// Scope a daemon session's gates without changing process-global environment.
pub fn with_context<T>(context: GateContext, operation: impl FnOnce() -> T) -> T {
    struct Restore(Option<GateContext>);

    impl Drop for Restore {
        fn drop(&mut self) {
            CONTEXT.with(|context| *context.borrow_mut() = self.0.take());
        }
    }

    let previous = CONTEXT.with(|slot| slot.borrow_mut().replace(context));
    let _restore = Restore(previous);

    operation()
}

/// Deterministic gates, linked only when the integration-test-hooks feature is enabled.
pub fn checkpoint(name: &str, cancelled: &AtomicBool) -> Result<(), String> {
    let context = CONTEXT.with(|slot| slot.borrow().clone());
    let directory = if let Some(context) = context {
        let Some(directory) = context.select(name)? else {
            return Ok(());
        };

        directory
    } else {
        if env::var("ANYQ_TEST_CHECKPOINT").ok().as_deref() != Some(name) {
            return Ok(());
        }

        let target = env::var("ANYQ_TEST_CHECKPOINT_HIT")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(1);
        if HITS.fetch_add(1, Ordering::SeqCst) + 1 != target {
            return Ok(());
        }

        PathBuf::from(
            env::var_os("ANYQ_TEST_CHECKPOINT_DIR").ok_or("checkpoint directory is missing")?,
        )
    };
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    fs::write(directory.join("ready"), name).map_err(|error| error.to_string())?;

    while !directory.join("release").exists() && !cancelled.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_millis(10));
    }

    Ok(())
}
