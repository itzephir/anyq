use crate::Stream;
use interprocess::os::windows::security_descriptor::SecurityDescriptor;
use std::{
    io,
    os::windows::io::{AsHandle, AsRawHandle},
};
use windows_permissions::{
    Sid,
    constants::{SeObjectType::SE_FILE_OBJECT, SecurityInformation},
    utilities::current_process_sid,
    wrappers::GetSecurityInfo,
};

pub(crate) fn descriptor() -> io::Result<SecurityDescriptor> {
    let user = current_process_sid()?;
    let sddl = widestring::U16CString::from_str(format!("O:{user}D:P(A;;GA;;;OW)"))
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;

    SecurityDescriptor::deserialize(&sddl)
}

fn validate_owner(handle: &impl AsRawHandle, expected: &Sid) -> io::Result<()> {
    let security = GetSecurityInfo(handle, SE_FILE_OBJECT, SecurityInformation::Owner)?;
    if security.owner() != Some(expected) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "daemon pipe belongs to another account",
        ));
    }

    Ok(())
}

pub(crate) fn validate(stream: &Stream) -> io::Result<()> {
    let interprocess::local_socket::tokio::Stream::NamedPipe(pipe) = &stream.inner;
    let user = current_process_sid()?;

    validate_owner(&pipe.as_handle(), &user)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Endpoint;

    #[test]
    fn connected_pipe_owner_matches_the_current_account() {
        let directory = tempfile::tempdir().unwrap();
        let endpoint = Endpoint::resolve(&directory.path().join("queue.db")).unwrap();
        let _listener = endpoint.listen().unwrap();

        let stream = endpoint.connect().unwrap();

        validate(&stream).unwrap();
    }

    #[test]
    fn connected_pipe_rejects_a_different_expected_account() {
        let directory = tempfile::tempdir().unwrap();
        let endpoint = Endpoint::resolve(&directory.path().join("queue.db")).unwrap();
        let _listener = endpoint.listen().unwrap();
        let stream = endpoint.connect().unwrap();
        let interprocess::local_socket::tokio::Stream::NamedPipe(pipe) = &stream.inner;
        let other: windows_permissions::LocalBox<Sid> = "S-1-5-7".parse().unwrap();

        let error = validate_owner(&pipe.as_handle(), &other).unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
    }

    #[test]
    fn unreadable_pipe_owner_fails_closed() {
        use std::{fs::OpenOptions, os::windows::fs::OpenOptionsExt};

        // FILE_READ_DATA | FILE_WRITE_DATA | SYNCHRONIZE, without READ_CONTROL.
        const DATA_WITHOUT_READ_CONTROL: u32 = 0x0010_0003;

        let directory = tempfile::tempdir().unwrap();
        let endpoint = Endpoint::resolve(&directory.path().join("queue.db")).unwrap();
        let _listener = endpoint.listen().unwrap();
        let file = OpenOptions::new()
            .access_mode(DATA_WITHOUT_READ_CONTROL)
            .open(format!(r"\\.\pipe\anyq-{}", endpoint.identity))
            .unwrap();
        let user = current_process_sid().unwrap();

        let error = validate_owner(&file, &user).unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
    }
}
