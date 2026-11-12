use crate::{Listener, OwnerLock, Stream, io_runtime::io_runtime};
use interprocess::local_socket::{
    GenericFilePath, GenericNamespaced, ListenerOptions, Name, prelude::*,
    traits::tokio::Stream as _,
};
use sha2::{Digest, Sha256};
use std::{
    fs, io,
    path::{Component, Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct Endpoint {
    pub database: PathBuf,
    pub identity: String,
    address: PathBuf,
}

fn canonical_database(path: &Path) -> io::Result<PathBuf> {
    fn resolve(path: &Path) -> io::Result<PathBuf> {
        match path.canonicalize() {
            Ok(path) => return Ok(path),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }

        if fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
            let target = fs::read_link(path)?;
            return resolve(&if target.is_absolute() {
                target
            } else {
                path.parent()
                    .ok_or_else(|| io::Error::other("invalid symlink parent"))?
                    .join(target)
            });
        }

        let parent = path
            .parent()
            .ok_or_else(|| io::Error::other("database has no existing parent"))?;
        let mut resolved = resolve(parent)?;
        match path.components().next_back() {
            Some(Component::ParentDir) => {
                resolved.pop();
            }
            Some(Component::CurDir) => {}
            Some(Component::Normal(name)) => resolved.push(name),
            _ => return Err(io::Error::other("invalid database path")),
        }

        Ok(resolved)
    }

    let absolute = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()?.join(path)
    };

    let database = resolve(&absolute)?;
    if database.try_exists()? {
        return Ok(database);
    }

    // Before SQLite exists, the regular owner sidecar anchors native spelling
    // on case-insensitive filesystems. Discovery itself must not create a file.
    let lock = OwnerLock::path(&database);
    match fs::symlink_metadata(&lock) {
        Ok(metadata) if metadata.is_file() => {
            let canonical = lock.canonicalize()?;
            let anchored = canonical.with_extension("").with_extension("");
            if canonical.parent() == database.parent() && OwnerLock::path(&anchored) == canonical {
                return Ok(anchored);
            }
        }
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }

    Ok(database)
}

impl Endpoint {
    pub fn resolve(path: &Path) -> io::Result<Self> {
        let database = canonical_database(path)?;
        let identity = format!(
            "{:x}",
            Sha256::digest(database.as_os_str().as_encoded_bytes())
        );

        #[cfg(unix)]
        let address = PathBuf::from(format!(
            "/tmp/anyq-{}/{}.sock",
            rustix::process::geteuid().as_raw(),
            identity
        ));

        #[cfg(windows)]
        let address = PathBuf::from(format!("anyq-{identity}"));

        Ok(Self {
            database,
            identity,
            address,
        })
    }

    fn name(&self) -> io::Result<Name<'_>> {
        if cfg!(windows) {
            self.address.as_os_str().to_ns_name::<GenericNamespaced>()
        } else {
            self.address.as_os_str().to_fs_name::<GenericFilePath>()
        }
    }

    pub fn connect(&self) -> io::Result<Stream> {
        let inner = io_runtime()?.block_on(async {
            tokio::time::timeout(
                std::time::Duration::from_secs(5),
                interprocess::local_socket::tokio::Stream::connect(self.name()?),
            )
            .await
            .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "daemon connection timed out"))?
        })?;
        let stream = Stream { inner };
        validate_peer(&stream)?;

        Ok(stream)
    }

    pub fn listen(&self) -> io::Result<Listener> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::{DirBuilderExt, FileTypeExt, MetadataExt, PermissionsExt};

            let directory = self
                .address
                .parent()
                .ok_or_else(|| io::Error::other("missing runtime directory"))?;
            match fs::DirBuilder::new().mode(0o700).create(directory) {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                Err(error) => return Err(error),
            }

            let metadata = fs::symlink_metadata(directory)?;
            if !metadata.is_dir()
                || metadata.uid() != rustix::process::geteuid().as_raw()
                || metadata.permissions().mode() & 0o077 != 0
            {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "daemon runtime directory is not private",
                ));
            }

            if let Ok(metadata) = fs::symlink_metadata(&self.address) {
                if !metadata.file_type().is_socket()
                    || metadata.uid() != rustix::process::geteuid().as_raw()
                {
                    return Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "refusing to replace a non-owned socket",
                    ));
                }

                fs::remove_file(&self.address)?;
            }
        }

        let options = ListenerOptions::new().name(self.name()?);

        #[cfg(windows)]
        let options = {
            use interprocess::os::windows::local_socket::ListenerOptionsExt;

            options.security_descriptor(crate::windows_peer::descriptor()?)
        };

        let _runtime = io_runtime()?.enter();
        let inner = options.create_tokio()?;
        let listener = Listener {
            inner: std::sync::Arc::new(inner),
            pending: std::sync::Mutex::new(None),
        };

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            fs::set_permissions(&self.address, fs::Permissions::from_mode(0o600))?;
        }

        Ok(listener)
    }

    pub fn lock_path(&self) -> PathBuf {
        OwnerLock::path(&self.database)
    }
}

pub fn validate_peer(stream: &Stream) -> io::Result<()> {
    #[cfg(unix)]
    if stream.peer_creds()?.euid() != Some(rustix::process::geteuid().as_raw()) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "daemon peer belongs to another account",
        ));
    }

    #[cfg(windows)]
    crate::windows_peer::validate(stream)?;

    Ok(())
}
