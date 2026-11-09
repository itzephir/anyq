use std::{
    env, fs,
    path::PathBuf,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    thread,
    time::Duration,
};

static HITS: AtomicUsize = AtomicUsize::new(0);

/// Deterministic gates, linked only when the integration-test-hooks feature is enabled.
pub fn checkpoint(name: &str, cancelled: &AtomicBool) -> Result<(), String> {
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

    let directory = PathBuf::from(
        env::var_os("ANYQ_TEST_CHECKPOINT_DIR").ok_or("checkpoint directory is missing")?,
    );
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    fs::write(directory.join("ready"), name).map_err(|error| error.to_string())?;

    while !directory.join("release").exists() && !cancelled.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_millis(10));
    }

    Ok(())
}
