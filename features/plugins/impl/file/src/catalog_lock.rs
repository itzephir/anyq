use anyq_queue_api::{Error, Result};
use std::{
    fs::{File, OpenOptions},
    io::ErrorKind,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::Duration,
};

pub(crate) struct CatalogLock {
    _file: File,
}

impl CatalogLock {
    pub fn acquire(path: &Path, cancelled: &AtomicBool) -> Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(path)
            .map_err(|error| Error::Storage(error.to_string()))?;

        loop {
            if cancelled.load(Ordering::SeqCst) {
                return Err(Error::Interrupted);
            }
            match fs2::FileExt::try_lock_exclusive(&file) {
                Ok(()) => return Ok(Self { _file: file }),
                Err(error) if error.kind() == ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(error) => return Err(Error::Storage(error.to_string())),
            }
        }
    }
}
