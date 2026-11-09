use crate::{
    checkpoint::checkpoint,
    codec::{decode, encode},
    file_lock::FileLock,
};
use anyq_config_api::{Config, Error, Result};
use anyq_config_dispatch::ports::Store;
use std::{
    fs,
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
    sync::{Arc, atomic::AtomicBool},
};

#[derive(Clone)]
pub struct FileConfig {
    path: PathBuf,
    cancelled: Arc<AtomicBool>,
}

impl FileConfig {
    pub fn new(path: &Path) -> Self {
        Self::with_cancellation(path, Arc::new(AtomicBool::new(false)))
    }

    pub fn with_cancellation(path: &Path, cancelled: Arc<AtomicBool>) -> Self {
        Self {
            path: path.to_owned(),
            cancelled,
        }
    }
}

impl Store for FileConfig {
    fn read(&mut self) -> Result<Config> {
        checkpoint("config.before-read", &self.cancelled)?;
        let config = match fs::read_to_string(&self.path) {
            Ok(input) => decode(&input)?,
            Err(error) if error.kind() == ErrorKind::NotFound => Config::default(),
            Err(error) => return Err(Error::Storage(error.to_string())),
        };
        checkpoint("config.after-read", &self.cancelled)?;

        Ok(config)
    }

    fn change<T>(&mut self, operation: impl FnOnce(&mut Config) -> Result<T>) -> Result<T> {
        checkpoint("config.before-change", &self.cancelled)?;
        let parent = self
            .path
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or(Path::new("."))
            .to_owned();
        fs::create_dir_all(&parent).map_err(|error| Error::Storage(error.to_string()))?;
        let _lock = FileLock::acquire(&self.path.with_extension("toml.lock"), &self.cancelled)?;
        let mut config = self.read()?;
        let result = operation(&mut config)?;
        let contents = encode(&config)?;
        let mut temporary = tempfile::NamedTempFile::new_in(&parent)
            .map_err(|error| Error::Storage(error.to_string()))?;
        temporary
            .write_all(contents.as_bytes())
            .map_err(|error| Error::Storage(error.to_string()))?;
        temporary
            .as_file()
            .sync_all()
            .map_err(|error| Error::Storage(error.to_string()))?;
        checkpoint("config.before-replace", &self.cancelled)?;
        temporary
            .persist(&self.path)
            .map_err(|error| Error::Storage(error.error.to_string()))?;
        checkpoint("config.after-replace", &self.cancelled)?;

        Ok(result)
    }
}
