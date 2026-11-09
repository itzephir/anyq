use crate::checkpoint::checkpoint;
use anyq_config_api::{Error, Result};
use fs2::FileExt;
use std::{
    fs::{File, OpenOptions},
    path::Path,
    sync::atomic::AtomicBool,
    thread,
    time::{Duration, Instant},
};

pub(crate) struct FileLock {
    file: File,
}

impl FileLock {
    pub fn acquire(path: &Path, cancelled: &AtomicBool) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)
            .map_err(|error| Error::Storage(error.to_string()))?;
        let deadline = Instant::now() + Duration::from_secs(5);

        loop {
            checkpoint("config.before-lock", cancelled)?;

            match FileExt::try_lock_exclusive(&file) {
                Ok(()) => return Ok(Self { file }),
                Err(error)
                    if error.raw_os_error() == fs2::lock_contended_error().raw_os_error()
                        && Instant::now() < deadline =>
                {
                    checkpoint("config.waiting-lock", cancelled)?;
                    thread::sleep(Duration::from_millis(10));
                }
                Err(error) => return Err(Error::Storage(error.to_string())),
            }
        }
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.file);
    }
}
