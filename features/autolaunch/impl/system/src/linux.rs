use crate::{NativeRegistration, registration_file};
use anyq_queue_api::{Error, Result};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

fn path(registration: &NativeRegistration) -> PathBuf {
    registration.config_directory.join("autostart/anyq.desktop")
}

pub(crate) fn signature(launcher: &Path) -> Result<String> {
    let mut escaped = String::new();
    for character in launcher
        .to_str()
        .ok_or_else(|| Error::Invalid("launcher path is not Unicode".into()))?
        .chars()
    {
        match character {
            '\\' => escaped.push_str("\\\\\\\\"),
            '"' => escaped.push_str("\\\\\""),
            '$' => escaped.push_str("\\\\$"),
            '`' => escaped.push_str("\\\\`"),
            '%' => escaped.push_str("%%"),
            '\n' | '\r' | '\0' => {
                return Err(Error::Invalid(
                    "launcher path contains a control character".into(),
                ));
            }
            _ => escaped.push(character),
        }
    }

    Ok(format!("\"{escaped}\" daemon start"))
}

pub(crate) fn read(registration: &NativeRegistration) -> Result<Option<String>> {
    let Some(bytes) = registration_file::read(&path(registration))? else {
        return Ok(None);
    };
    let text = String::from_utf8(bytes).map_err(|error| Error::Invalid(error.to_string()))?;
    let mut active = false;
    let mut fields = BTreeMap::new();
    for line in text.lines().map(str::trim) {
        if line.starts_with('[') {
            active = line == "[Desktop Entry]";
            continue;
        }
        if active && !line.is_empty() && !line.starts_with('#') {
            let (key, value) = line
                .split_once('=')
                .ok_or_else(|| Error::Invalid("invalid system autostart entry".into()))?;
            if fields.insert(key, value).is_some() {
                return Err(Error::Invalid("duplicate system autostart field".into()));
            }
        }
    }
    if fields.get("Hidden") == Some(&"true")
        || fields.get("X-GNOME-Autostart-enabled") == Some(&"false")
    {
        return Ok(None);
    }
    if fields.get("Type") != Some(&"Application") {
        return Err(Error::Invalid(
            "unsupported system autostart entry type".into(),
        ));
    }

    fields
        .get("Exec")
        .map(|value| Some((*value).to_owned()))
        .ok_or_else(|| Error::Invalid("system autostart entry has no command".into()))
}

pub(crate) fn publish(registration: &NativeRegistration, launcher: &Path) -> Result<()> {
    let contents = format!(
        "[Desktop Entry]\nType=Application\nName=anyq\nExec={}\nTerminal=false\n",
        signature(launcher)?
    );
    registration_file::publish(&path(registration), contents.as_bytes())
}

pub(crate) fn remove(registration: &NativeRegistration) -> Result<()> {
    registration_file::remove(&path(registration))
}
