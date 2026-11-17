use anyq_queue_api::{Error, Result};
use std::{
    fs,
    io::{ErrorKind, Read, Write},
    path::Path,
};

pub(crate) fn read(path: &Path) -> Result<Option<Vec<u8>>> {
    let file = match fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(Error::Storage(error.to_string())),
    };
    let mut bytes = Vec::new();
    file.take(1_048_577)
        .read_to_end(&mut bytes)
        .map_err(|error| Error::Storage(error.to_string()))?;
    if bytes.len() > 1_048_576 {
        return Err(Error::Invalid(
            "system autolaunch registration exceeds 1 MiB".into(),
        ));
    }

    Ok(Some(bytes))
}

pub(crate) fn publish(path: &Path, contents: &[u8]) -> Result<()> {
    let directory = path
        .parent()
        .ok_or_else(|| Error::Invalid("system registration has no parent".into()))?;
    fs::create_dir_all(directory).map_err(|error| Error::Storage(error.to_string()))?;
    let mut temporary = tempfile::NamedTempFile::new_in(directory)
        .map_err(|error| Error::Storage(error.to_string()))?;
    temporary
        .write_all(contents)
        .map_err(|error| Error::Storage(error.to_string()))?;
    temporary
        .as_file()
        .sync_all()
        .map_err(|error| Error::Storage(error.to_string()))?;
    temporary
        .persist(path)
        .map_err(|error| Error::Storage(error.error.to_string()))?;

    Ok(())
}

pub(crate) fn remove(path: &Path) -> Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(Error::Storage(error.to_string())),
    }
}
