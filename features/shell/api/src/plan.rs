use crate::Source;
use anyq_queue_api::{Error, Metadata, MetadataValue, Result};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    pub source: Source,
    pub entries: Vec<String>,
    pub autocomplete: bool,
    pub cwd: Option<String>,
    pub interpreter: Option<Vec<String>>,
    pub environment: BTreeMap<String, String>,
}

impl Plan {
    pub fn from_task(metadata: &Metadata) -> Result<Option<Self>> {
        let MetadataValue::Object(task) = metadata.value() else {
            unreachable!()
        };
        let Some(shell) = task.get("shell") else {
            return Ok(None);
        };
        let MetadataValue::Object(shell) = shell else {
            return Err(Error::Invalid("shell must be an object".into()));
        };
        if shell.keys().any(|key| {
            !["sh", "src", "autocomplete", "cwd", "interpreter", "env"].contains(&key.as_str())
        }) {
            return Err(Error::Invalid("unknown shell option".into()));
        }
        let (source, entries) = match (shell.get("sh"), shell.get("src")) {
            (Some(entries), None) => (Source::Shell, strings(entries)?),
            (None, Some(entries)) => (Source::Executable, strings(entries)?),
            _ => {
                return Err(Error::Invalid(
                    "shell requires exactly one nonempty sh or src array".into(),
                ));
            }
        };
        if entries.len() > 128 {
            return Err(Error::Invalid("shell entry limit is 128".into()));
        }
        let autocomplete = match shell.get("autocomplete") {
            None => false,
            Some(MetadataValue::Bool(value)) => *value,
            _ => return Err(Error::Invalid("shell.autocomplete must be boolean".into())),
        };
        let cwd = shell.get("cwd").map(text).transpose()?;
        let interpreter = shell
            .get("interpreter")
            .map(|value| match value {
                MetadataValue::String(_) => text(value).map(|value| vec![value]),
                _ => strings(value),
            })
            .transpose()?;
        let mut environment = BTreeMap::new();
        if let Some(value) = shell.get("env") {
            let MetadataValue::Object(values) = value else {
                return Err(Error::Invalid("shell.env must be a string object".into()));
            };
            for (key, value) in values {
                if key.is_empty() || key.contains(['=', '\0']) {
                    return Err(Error::Invalid("invalid shell environment key".into()));
                }
                let MetadataValue::String(value) = value else {
                    return Err(Error::Invalid("shell.env values must be strings".into()));
                };
                if value.contains('\0') {
                    return Err(Error::Invalid("shell.env contains a null byte".into()));
                }
                environment.insert(key.clone(), value.clone());
            }
        }

        Ok(Some(Self {
            source,
            entries,
            autocomplete,
            cwd,
            interpreter,
            environment,
        }))
    }
}

fn text(value: &MetadataValue) -> Result<String> {
    match value {
        MetadataValue::String(value) if !value.is_empty() && !value.contains('\0') => {
            Ok(value.clone())
        }
        _ => Err(Error::Invalid(
            "shell option must be a nonempty string without null bytes".into(),
        )),
    }
}

fn strings(value: &MetadataValue) -> Result<Vec<String>> {
    let MetadataValue::Array(values) = value else {
        return Err(Error::Invalid(
            "shell entries/interpreter must be a nonempty string array".into(),
        ));
    };
    if values.is_empty() {
        return Err(Error::Invalid("shell string array cannot be empty".into()));
    }

    values.iter().map(text).collect()
}
