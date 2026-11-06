use crate::{Error, Result};

pub fn validate_name(value: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
    {
        return Err(Error::Invalid(
            "names must contain 1..128 ASCII letters, digits, '-', '_' or '.'".into(),
        ));
    }

    Ok(())
}
