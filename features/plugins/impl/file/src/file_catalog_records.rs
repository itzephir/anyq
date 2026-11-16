use crate::{catalog_document::CatalogDocument, enablement_dto::EnablementDto};
use anyq_plugin_api::{InstalledPlugin, validate_catalog_name};
use anyq_plugin_dispatch::CatalogRecords;
use anyq_queue_api::{Error, Result};
use std::{
    collections::BTreeSet,
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

pub(crate) struct FileCatalogRecords {
    pub directory: PathBuf,
    pub document: CatalogDocument,
}

impl CatalogRecords for FileCatalogRecords {
    fn list(&self) -> Result<Vec<InstalledPlugin>> {
        let mut names: BTreeSet<String> = self.document.plugins.keys().cloned().collect();
        let entries = match fs::read_dir(&self.directory) {
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
                #[cfg(windows)]
                let Some(name) = name.strip_suffix(".exe") else {
                    continue;
                };
                if validate_catalog_name(name).is_ok() && executable_file(&entry.path())? {
                    names.insert(name.to_owned());
                }
            }
        }

        names
            .into_iter()
            .map(|name| {
                let file_name = format!("{name}{}", std::env::consts::EXE_SUFFIX);
                let path = self.directory.join(file_name);
                let available = executable_file(&path)?;
                let policy = self.document.plugins.get(&name);

                Ok(InstalledPlugin {
                    name,
                    executable: path
                        .to_str()
                        .ok_or_else(|| Error::Invalid("plugin path is not Unicode".into()))?
                        .into(),
                    available,
                    enabled: policy.is_some_and(|plugin| plugin.enabled),
                    revision: policy.map_or(0, |plugin| plugin.revision),
                })
            })
            .collect()
    }

    fn save(&mut self, plugin: &InstalledPlugin) -> Result<()> {
        validate_catalog_name(&plugin.name)?;
        self.document.plugins.insert(
            plugin.name.clone(),
            EnablementDto {
                enabled: plugin.enabled,
                revision: plugin.revision,
            },
        );

        self.document.validate()
    }
}

fn executable_file(path: &Path) -> Result<bool> {
    match fs::metadata(path) {
        Ok(metadata) => {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;

                Ok(metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
            }
            #[cfg(not(unix))]
            Ok(metadata.is_file())
        }
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(error) => Err(Error::Storage(error.to_string())),
    }
}
