use crate::error::{Error, Result};
use std::path::PathBuf;

pub fn resolve(value: &str) -> Result<String> {
    let path = PathBuf::from(value);
    let candidate = if path.is_absolute() || path.components().count() > 1 {
        std::env::current_dir()?.join(path)
    } else {
        let extensions: Vec<String> = if cfg!(windows) {
            std::env::var("PATHEXT")
                .unwrap_or_else(|_| ".EXE;.CMD;.BAT;.COM".into())
                .split(';')
                .map(str::to_string)
                .collect()
        } else {
            Vec::new()
        };
        std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())
            .flat_map(|directory| {
                let mut candidates = vec![directory.join(&path)];
                candidates.extend(
                    extensions
                        .iter()
                        .map(|extension| directory.join(format!("{value}{extension}"))),
                );
                candidates
            })
            .find(|candidate| candidate.is_file())
            .ok_or_else(|| Error::Invalid("plugin executable was not found in PATH".into()))?
    };
    let candidate = candidate.canonicalize()?;
    if !candidate.is_file() {
        return Err(Error::Invalid("plugin executable must be a file".into()));
    }

    candidate
        .into_os_string()
        .into_string()
        .map_err(|_| Error::Invalid("plugin executable path must be UTF-8".into()))
}
