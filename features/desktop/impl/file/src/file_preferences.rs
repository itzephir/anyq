use crate::codec::{decode, encode};
use anyq_desktop_api::{Error, PanelPreferences, Result};
use anyq_desktop_dispatch::ports::PreferenceStore;
use fs2::FileExt;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Clone)]
pub struct FilePreferences {
    path: PathBuf,
}

fn storage(error: impl ToString) -> Error {
    Error::Storage(error.to_string())
}

impl FilePreferences {
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_owned(),
        }
    }
}

impl PreferenceStore for FilePreferences {
    fn load(&mut self) -> Result<PanelPreferences> {
        match fs::read_to_string(&self.path) {
            Ok(text) => decode(&text),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                Ok(PanelPreferences::default())
            }
            Err(error) => Err(storage(error)),
        }
    }

    fn save(&mut self, preferences: &PanelPreferences) -> Result<()> {
        let content = encode(preferences)?;
        let parent = self
            .path
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .map(Path::to_owned)
            .ok_or_else(|| Error::Invalid("panel preference path has no parent".into()))?;
        fs::create_dir_all(&parent).map_err(storage)?;
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(self.path.with_extension("toml.lock"))
            .map_err(storage)?;
        lock.lock_exclusive().map_err(storage)?;

        // A malformed newer schema must never be silently replaced by defaults.
        self.load()?;
        let mut temporary = tempfile::NamedTempFile::new_in(&parent).map_err(storage)?;
        temporary.write_all(content.as_bytes()).map_err(storage)?;
        temporary.as_file().sync_all().map_err(storage)?;
        temporary.persist(&self.path).map_err(storage)?;

        Ok(())
    }
}
