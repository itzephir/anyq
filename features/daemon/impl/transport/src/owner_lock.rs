use crate::Endpoint;
use fs2::FileExt;
use std::{
    fs::{self, File, OpenOptions},
    io,
};

pub struct OwnerLock {
    _file: File,
}

impl OwnerLock {
    pub fn is_held(endpoint: &Endpoint) -> io::Result<bool> {
        let file = match OpenOptions::new()
            .read(true)
            .write(true)
            .open(endpoint.lock_path())
        {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
            Err(error) => return Err(error),
        };

        match file.try_lock_exclusive() {
            Ok(()) => Ok(false),
            Err(error) if error.raw_os_error() == fs2::lock_contended_error().raw_os_error() => {
                Ok(true)
            }
            Err(error) => Err(error),
        }
    }

    pub fn acquire(endpoint: &Endpoint) -> io::Result<Self> {
        if let Some(parent) = endpoint.database.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut options = OpenOptions::new();
        options.read(true).write(true).create(true).truncate(false);

        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;

            options.mode(0o600);
        }

        let file = options.open(endpoint.lock_path())?;
        file.try_lock_exclusive()?;

        Ok(Self { _file: file })
    }
}
