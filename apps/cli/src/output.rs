use crate::error::Result;
use std::io::Write;

pub fn emit(message: &str) -> Result<()> {
    writeln!(std::io::stdout().lock(), "{message}")?;

    Ok(())
}
