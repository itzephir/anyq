use crate::{
    catalog_document::CatalogDocument, catalog_lock::CatalogLock,
    file_catalog_records::FileCatalogRecords,
};
use anyq_plugin_api::InstalledPlugin;
use anyq_plugin_dispatch::{CatalogRecords, CatalogStore};
use anyq_queue_api::{Error, Result};
use std::{
    fs,
    io::{ErrorKind, Read, Write},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

#[derive(Clone)]
pub struct FileCatalog {
    directory: PathBuf,
    cancelled: Arc<AtomicBool>,
}

impl FileCatalog {
    pub fn new(directory: &Path, cancelled: Arc<AtomicBool>) -> Result<Self> {
        Ok(Self {
            directory: std::path::absolute(directory)
                .map_err(|error| Error::Storage(error.to_string()))?,
            cancelled,
        })
    }

    fn records(&self) -> Result<FileCatalogRecords> {
        if self.cancelled.load(Ordering::SeqCst) {
            return Err(Error::Interrupted);
        }
        let mut contents = String::new();
        let exists = match fs::File::open(self.directory.join(".catalog.toml")) {
            Ok(file) => {
                file.take(1_048_577)
                    .read_to_string(&mut contents)
                    .map_err(|error| Error::Storage(error.to_string()))?;
                if contents.len() > 1_048_576 {
                    return Err(Error::Invalid("plugin catalog exceeds 1 MiB".into()));
                }
                true
            }
            Err(error) if error.kind() == ErrorKind::NotFound => false,
            Err(error) => return Err(Error::Storage(error.to_string())),
        };
        let document = if exists {
            toml::from_str::<CatalogDocument>(&contents)
                .map_err(|error| Error::Invalid(format!("invalid plugin catalog: {error}")))?
        } else {
            CatalogDocument::default()
        };
        document.validate()?;

        Ok(FileCatalogRecords {
            directory: self.directory.clone(),
            document,
        })
    }
}

impl CatalogStore for FileCatalog {
    fn read(&mut self) -> Result<Vec<InstalledPlugin>> {
        self.records()?.list()
    }

    fn change<T>(
        &mut self,
        operation: impl FnOnce(&mut dyn CatalogRecords) -> Result<T>,
    ) -> Result<T> {
        fs::create_dir_all(&self.directory).map_err(|error| Error::Storage(error.to_string()))?;
        let _lock = CatalogLock::acquire(&self.directory.join(".catalog.lock"), &self.cancelled)?;
        let mut records = self.records()?;

        let result = operation(&mut records)?;
        records.document.validate()?;
        let contents = toml::to_string(&records.document)
            .map_err(|error| Error::Storage(error.to_string()))?;
        let mut temporary = tempfile::NamedTempFile::new_in(&self.directory)
            .map_err(|error| Error::Storage(error.to_string()))?;
        temporary
            .write_all(contents.as_bytes())
            .map_err(|error| Error::Storage(error.to_string()))?;
        temporary
            .as_file()
            .sync_all()
            .map_err(|error| Error::Storage(error.to_string()))?;
        if self.cancelled.load(Ordering::SeqCst) {
            return Err(Error::Interrupted);
        }
        temporary
            .persist(self.directory.join(".catalog.toml"))
            .map_err(|error| Error::Storage(error.error.to_string()))?;

        Ok(result)
    }
}
