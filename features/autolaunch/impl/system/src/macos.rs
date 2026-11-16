use crate::{NativeRegistration, registration_file};
use anyq_queue_api::{Error, Result};
use plist::{Dictionary, Value};
use std::{
    io::Cursor,
    path::{Path, PathBuf},
};

const LABEL: &str = "com.itzephir.anyq.autolaunch";

fn path(registration: &NativeRegistration) -> PathBuf {
    registration
        .home
        .join("Library/LaunchAgents")
        .join(format!("{LABEL}.plist"))
}

pub(crate) fn signature(launcher: &Path) -> Result<String> {
    serde_json::to_string(&[
        launcher
            .to_str()
            .ok_or_else(|| Error::Invalid("launcher path is not Unicode".into()))?,
        "daemon",
        "start",
    ])
    .map_err(|error| Error::Storage(error.to_string()))
}

pub(crate) fn read(registration: &NativeRegistration) -> Result<Option<String>> {
    let Some(bytes) = registration_file::read(&path(registration))? else {
        return Ok(None);
    };
    let value = Value::from_reader(Cursor::new(bytes)).map_err(|error| {
        Error::Invalid(format!("invalid system autolaunch registration: {error}"))
    })?;
    let dictionary = value.as_dictionary().ok_or_else(|| {
        Error::Invalid("system autolaunch registration is not a property list dictionary".into())
    })?;
    if dictionary.get("Label").and_then(Value::as_string) != Some(LABEL) {
        return Err(Error::Invalid(
            "system autolaunch registration has another label".into(),
        ));
    }
    if dictionary.get("Disabled").and_then(Value::as_boolean) == Some(true)
        || dictionary.get("RunAtLoad").and_then(Value::as_boolean) != Some(true)
    {
        return Ok(None);
    }
    let arguments = dictionary
        .get("ProgramArguments")
        .and_then(Value::as_array)
        .ok_or_else(|| Error::Invalid("system autolaunch has no program arguments".into()))?;
    let arguments = arguments
        .iter()
        .map(|argument| {
            argument
                .as_string()
                .ok_or_else(|| Error::Invalid("invalid system launch argument".into()))
        })
        .collect::<Result<Vec<_>>>()?;

    serde_json::to_string(&arguments)
        .map(Some)
        .map_err(|error| Error::Storage(error.to_string()))
}

pub(crate) fn publish(registration: &NativeRegistration, launcher: &Path) -> Result<()> {
    let mut dictionary = Dictionary::new();
    dictionary.insert("Label".into(), Value::String(LABEL.into()));
    dictionary.insert("RunAtLoad".into(), Value::Boolean(true));
    dictionary.insert("AbandonProcessGroup".into(), Value::Boolean(true));
    dictionary.insert(
        "ProgramArguments".into(),
        Value::Array(vec![
            Value::String(
                launcher
                    .to_str()
                    .ok_or_else(|| Error::Invalid("launcher path is not Unicode".into()))?
                    .into(),
            ),
            Value::String("daemon".into()),
            Value::String("start".into()),
        ]),
    );
    let mut contents = Vec::new();
    Value::Dictionary(dictionary)
        .to_writer_xml(&mut contents)
        .map_err(|error| Error::Storage(error.to_string()))?;

    registration_file::publish(&path(registration), &contents)
}

pub(crate) fn remove(registration: &NativeRegistration) -> Result<()> {
    registration_file::remove(&path(registration))
}
