use crate::{Error, Result, validate_name};

pub fn validate_topic_name(name: &str) -> Result<()> {
    validate_name(name)?;

    if name.starts_with('-') {
        return Err(Error::Invalid("topic names must not start with '-'".into()));
    }

    if matches!(name, "create" | "capacity" | "link" | "help") {
        return Err(Error::Invalid(format!(
            "topic name '{name}' is reserved for a command"
        )));
    }

    Ok(())
}
