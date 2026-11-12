use crate::{RuntimeOpen, Sqlite, storage_error::storage, validate_runtime::validate_runtime};
use anyq_queue_api::{Error, Result};
use rusqlite::{Connection, OpenFlags};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

fn io_error(error: impl std::fmt::Display) -> Error {
    Error::Storage(error.to_string())
}

fn sidecar(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(suffix);

    name.into()
}

fn copy(source: &Path, target: &Path, shutdown: &AtomicBool) -> Result<()> {
    let mut input = File::open(source).map_err(io_error)?;
    let mut output = File::create(target).map_err(io_error)?;
    let mut buffer = [0; 64 * 1024];

    loop {
        if shutdown.load(Ordering::SeqCst) {
            return Err(Error::Interrupted);
        }
        let count = input.read(&mut buffer).map_err(io_error)?;
        if count == 0 {
            return Ok(());
        }
        output.write_all(&buffer[..count]).map_err(io_error)?;
    }
}

fn inspect(
    connection: &Connection,
    validate: fn(&Connection) -> Result<()>,
) -> Result<Option<RuntimeOpen>> {
    let read = connection.unchecked_transaction().map_err(storage)?;
    let outcome = inspect_schema(&read, validate);
    read.commit().map_err(storage)?;

    outcome
}

fn inspect_schema(
    connection: &Connection,
    validate: fn(&Connection) -> Result<()>,
) -> Result<Option<RuntimeOpen>> {
    let version: i64 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(storage)?;
    if version == 0 {
        return Ok(Some(RuntimeOpen::Uninitialized));
    }
    if [2, 3, 4, 5].contains(&version) {
        return Ok(Some(RuntimeOpen::UpgradeRequired(version)));
    }
    if version != 6 {
        return Ok(Some(RuntimeOpen::Incompatible(format!(
            "unsupported runtime schema {version}"
        ))));
    }

    if let Err(error) = validate_runtime(connection).and_then(|()| validate(connection)) {
        return Ok(Some(RuntimeOpen::Incompatible(error.to_string())));
    }

    Ok(None)
}

pub(crate) fn open(
    path: &Path,
    shutdown: Arc<AtomicBool>,
    validate: fn(&Connection) -> Result<()>,
) -> Result<RuntimeOpen> {
    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(RuntimeOpen::Missing);
        }
        Err(error) => return Err(io_error(error)),
    };
    if !metadata.is_file() {
        return Ok(RuntimeOpen::Incompatible(
            "runtime database is not a regular file".into(),
        ));
    }
    if metadata.len() == 0 {
        return Ok(RuntimeOpen::Uninitialized);
    }

    // Validation may recover a crash-persisted WAL, but must not create or
    // recover sidecars on the selected database before its schema is known.
    let temporary = tempfile::tempdir().map_err(io_error)?;
    let snapshot = temporary.path().join("queue.db");
    copy(path, &snapshot, &shutdown)?;
    let wal = sidecar(path, "-wal");
    if wal.try_exists().map_err(io_error)? {
        copy(&wal, &sidecar(&snapshot, "-wal"), &shutdown)?;
    }
    let flags = OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let validation = Connection::open_with_flags(&snapshot, flags).map_err(storage)?;
    validation
        .pragma_update(None, "query_only", true)
        .map_err(storage)?;
    match inspect(&validation, validate) {
        Ok(Some(outcome)) => return Ok(outcome),
        Err(error) => return Ok(RuntimeOpen::Incompatible(error.to_string())),
        Ok(None) => {}
    }
    drop(validation);

    if shutdown.load(Ordering::SeqCst) {
        return Err(Error::Interrupted);
    }
    let connection = Connection::open_with_flags(path, flags).map_err(storage)?;
    match inspect(&connection, validate) {
        Ok(Some(outcome)) => return Ok(outcome),
        Err(error) => return Ok(RuntimeOpen::Incompatible(error.to_string())),
        Ok(None) => {}
    }

    Sqlite::from_runtime_connection(connection, shutdown).map(RuntimeOpen::Ready)
}
