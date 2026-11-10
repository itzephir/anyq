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
