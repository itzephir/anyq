use crate::NativeRegistration;
use anyq_queue_api::{Error, Result};
use sha2::{Digest, Sha256};
use std::{io::ErrorKind, path::Path};
use winreg::{
    RegKey,
    enums::{HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE},
};

const RUN: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";

fn name(registration: &NativeRegistration) -> String {
    format!(
        "anyq-{:x}",
        Sha256::digest(registration.home.as_os_str().as_encoded_bytes())
    )
}

pub(crate) fn signature(launcher: &Path) -> Result<String> {
    let path = launcher
        .to_str()
        .ok_or_else(|| Error::Invalid("launcher path is not Unicode".into()))?;
    if path.contains(['"', '\n', '\r', '\0']) {
        return Err(Error::Invalid("invalid Windows launcher path".into()));
    }

    Ok(format!("\"{path}\" daemon start"))
}

pub(crate) fn read(registration: &NativeRegistration) -> Result<Option<String>> {
    let key = match RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(RUN, KEY_READ) {
        Ok(key) => key,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(Error::Storage(error.to_string())),
    };
    match key.get_value::<String, _>(name(registration)) {
        Ok(value) => Ok(Some(value)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(Error::Storage(error.to_string())),
    }
}

pub(crate) fn publish(registration: &NativeRegistration, launcher: &Path) -> Result<()> {
    let command = signature(launcher)?;
    if command.encode_utf16().count() > 260 {
        return Err(Error::Invalid(
            "Windows login launch command exceeds 260 characters".into(),
        ));
    }
    let (key, _) = RegKey::predef(HKEY_CURRENT_USER)
        .create_subkey_with_flags(RUN, KEY_SET_VALUE)
        .map_err(|error| Error::Storage(error.to_string()))?;
    key.set_value(name(registration), &command)
        .map_err(|error| Error::Storage(error.to_string()))
}

pub(crate) fn remove(registration: &NativeRegistration) -> Result<()> {
    let key = match RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(RUN, KEY_SET_VALUE) {
        Ok(key) => key,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(Error::Storage(error.to_string())),
    };
    match key.delete_value(name(registration)) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(Error::Storage(error.to_string())),
    }
}
