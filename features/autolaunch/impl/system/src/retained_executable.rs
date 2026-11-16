use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::{self, Read, Seek},
    path::{Path, PathBuf},
};

pub fn retain(home: &Path) -> io::Result<PathBuf> {
    retain_source(&std::env::current_exe()?, home)
}

pub(crate) fn retain_source(executable: &Path, home: &Path) -> io::Result<PathBuf> {
    let mut source = File::open(executable)?;
    let digest = hash(&mut source)?;
    let directory = home.join(".anyq/owners").join(format!("{digest:x}"));
    fs::create_dir_all(&directory)?;
    let target = directory.join(if cfg!(windows) {
        "anyq-owner.exe"
    } else {
        "anyq-owner"
    });

    if !target.exists() {
        source.rewind()?;
        let mut temporary = tempfile::NamedTempFile::new_in(&directory)?;
        io::copy(&mut source, temporary.as_file_mut())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            temporary
                .as_file()
                .set_permissions(fs::Permissions::from_mode(0o700))?;
        }
        temporary.as_file().sync_all()?;
        match temporary.persist_noclobber(&target) {
            Ok(_) => {}
            Err(error) if error.error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error.error),
        }
    }

    if hash(&mut File::open(&target)?)? != digest {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "retained owner executable does not match the panel build",
        ));
    }

    Ok(target)
}

fn hash(source: &mut File) -> io::Result<sha2::digest::Output<Sha256>> {
    let mut digest = Sha256::new();
    let mut bytes = [0_u8; 65_536];
    loop {
        let count = source.read(&mut bytes)?;
        if count == 0 {
            return Ok(digest.finalize());
        }
        digest.update(&bytes[..count]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn concurrent_retention_publishes_one_complete_executable() {
        let home = tempfile::tempdir().unwrap();

        let paths = std::thread::scope(|scope| {
            let first = scope.spawn(|| retain(home.path()).unwrap());
            let second = scope.spawn(|| retain(home.path()).unwrap());
            [first.join().unwrap(), second.join().unwrap()]
        });

        assert_eq!(paths[0], paths[1]);
        assert_eq!(
            fs::read(&paths[0]).unwrap(),
            fs::read(std::env::current_exe().unwrap()).unwrap()
        );
        assert_eq!(fs::read_dir(paths[0].parent().unwrap()).unwrap().count(), 1);
    }

    #[test]
    fn altered_retained_bytes_are_rejected_without_overwriting() {
        let home = tempfile::tempdir().unwrap();
        let path = retain(home.path()).unwrap();
        fs::write(&path, b"invalid cached executable").unwrap();

        let error = retain(home.path()).unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert_eq!(fs::read(path).unwrap(), b"invalid cached executable");
    }

    #[test]
    fn retained_executable_runs_after_its_source_installation_is_removed() {
        let home = tempfile::tempdir().unwrap();
        let installation = home.path().join("Installed panel");
        fs::create_dir_all(&installation).unwrap();
        let source = installation.join("panel");
        fs::copy(std::env::current_exe().unwrap(), &source).unwrap();
        let retained = retain_source(&source, home.path()).unwrap();

        fs::remove_dir_all(installation).unwrap();
        let result = std::process::Command::new(&retained)
            .arg("--list")
            .output()
            .unwrap();

        assert!(result.status.success());
        assert!(
            String::from_utf8(result.stdout)
                .unwrap()
                .contains("retained_executable_runs_after_its_source_installation_is_removed")
        );
        assert!(retained.is_file());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            assert_eq!(
                fs::metadata(retained).unwrap().permissions().mode() & 0o777,
                0o700
            );
        }
    }
}
