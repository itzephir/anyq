use crate::{Error, Result};

pub fn parse_name(input: &str) -> Result<String> {
    if let Some(value) = input
        .strip_prefix('`')
        .and_then(|value| value.strip_suffix('`'))
    {
        return Ok(value.into());
    }

    if input
        .as_bytes()
        .first()
        .is_some_and(u8::is_ascii_alphabetic)
        && !matches!(
            input,
            "list"
                | "create"
                | "capacity"
                | "link"
                | "add"
                | "wait"
                | "complete"
                | "config"
                | "help"
        )
    {
        return Ok(input.into());
    }

    Err(Error::Invalid(
        "names must start with a Latin letter and not be reserved for a command, or be enclosed in literal backticks".into(),
    ))
}
