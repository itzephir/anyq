use anyq_queue_api::{Error, Result};
use std::{
    fs,
    io::ErrorKind,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};

pub fn install_missing_binary(
    source: &Path,
    destination: &Path,
    cancelled: &AtomicBool,
) -> Result<()> {
    if cancelled.load(Ordering::SeqCst) {
        return Err(Error::Interrupted);
    }
    if destination
        .try_exists()
        .map_err(|error| Error::Storage(error.to_string()))?
    {
        return Ok(());
    }
    let directory = destination
        .parent()
        .ok_or_else(|| Error::Invalid("plugin destination has no parent".into()))?;
    fs::create_dir_all(directory).map_err(|error| Error::Storage(error.to_string()))?;
    let temporary = tempfile::NamedTempFile::new_in(directory)
        .map_err(|error| Error::Storage(error.to_string()))?;
    fs::copy(source, temporary.path()).map_err(|error| Error::Storage(error.to_string()))?;
    temporary
        .as_file()
        .sync_all()
        .map_err(|error| Error::Storage(error.to_string()))?;
    if cancelled.load(Ordering::SeqCst) {
        return Err(Error::Interrupted);
    }
    match temporary.persist_noclobber(destination) {
        Ok(_) => Ok(()),
        Err(error) if error.error.kind() == ErrorKind::AlreadyExists => Ok(()),
        Err(error) => Err(Error::Storage(error.error.to_string())),
    }
}
