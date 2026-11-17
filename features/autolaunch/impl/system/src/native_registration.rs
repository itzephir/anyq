use anyq_autolaunch_dispatch::{ReadRegistration, WriteRegistration};
use anyq_queue_api::{Error, Result};
use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

#[derive(Clone)]
pub struct NativeRegistration {
    pub(crate) home: PathBuf,
    #[cfg(target_os = "linux")]
    pub(crate) config_directory: PathBuf,
    source: PathBuf,
    cancelled: Arc<AtomicBool>,
}

impl NativeRegistration {
    pub fn new(
        home: &Path,
        config_directory: Option<&Path>,
        source: &Path,
        cancelled: Arc<AtomicBool>,
    ) -> Result<Self> {
        let home = home
            .canonicalize()
            .map_err(|error| Error::Storage(error.to_string()))?;
        #[cfg(target_os = "linux")]
        let config_directory = config_directory
            .filter(|path| path.is_absolute())
            .map(Path::to_owned)
            .unwrap_or_else(|| home.join(".config"));
        #[cfg(not(target_os = "linux"))]
        let _ = config_directory;

        Ok(Self {
            home,
            #[cfg(target_os = "linux")]
            config_directory,
            source: source.to_owned(),
            cancelled,
        })
    }

    fn check_cancelled(&self) -> Result<()> {
        if self.cancelled.load(Ordering::SeqCst) {
            Err(Error::Interrupted)
        } else {
            Ok(())
        }
    }

    pub fn registered_command(&self) -> Result<Option<Vec<String>>> {
        self.check_cancelled()?;
        let Some(signature) = crate::platform::read(self)? else {
            return Ok(None);
        };
        let directory = self.home.join(".anyq/owners");
        let entries = match fs::read_dir(directory) {
            Ok(entries) => Some(entries),
            Err(error) if error.kind() == ErrorKind::NotFound => None,
            Err(error) => return Err(Error::Storage(error.to_string())),
        };
        if let Some(entries) = entries {
            for entry in entries {
                let entry = entry.map_err(|error| Error::Storage(error.to_string()))?;
                let file_name = entry.file_name();
                let Some(name) = file_name.to_str() else {
                    continue;
                };
                if name.len() != 64 || !name.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                    continue;
                }
                let launcher = entry.path().join(if cfg!(windows) {
                    "anyq-owner.exe"
                } else {
                    "anyq-owner"
                });
                if launcher.is_file() && crate::platform::signature(&launcher)? == signature {
                    return Ok(Some(vec![
                        launcher
                            .to_str()
                            .ok_or_else(|| Error::Invalid("launcher path is not Unicode".into()))?
                            .into(),
                        "daemon".into(),
                        "start".into(),
                    ]));
                }
            }
        }

        Err(Error::Invalid(
            "system autolaunch references an unavailable or unsupported launcher".into(),
        ))
    }
}

impl ReadRegistration for NativeRegistration {
    fn registered(&self) -> Result<bool> {
        Ok(self.registered_command()?.is_some())
    }
}

impl WriteRegistration for NativeRegistration {
    fn register(&self) -> Result<()> {
        self.check_cancelled()?;
        let launcher = crate::retained_executable::retain_source(&self.source, &self.home)
            .map_err(|error| Error::Storage(error.to_string()))?;
        self.check_cancelled()?;
        crate::platform::publish(self, &launcher)
    }

    fn unregister(&self) -> Result<()> {
        self.check_cancelled()?;
        crate::platform::remove(self)
    }
}
