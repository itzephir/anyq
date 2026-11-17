//! An example plugin-owned interpreter launcher, independent of anyq's catalog.
use std::{fs, io, path::PathBuf, process::{Command, ExitCode, Stdio}};

fn run() -> io::Result<i32> {
    let executable = std::env::current_exe()?;
    let settings = fs::read_to_string(executable.with_extension("launcher"))?;
    let mut lines = settings.lines();
    let interpreter = PathBuf::from(lines.next().ok_or_else(|| io::Error::other("missing interpreter"))?);
    let script = lines.next().ok_or_else(|| io::Error::other("missing script"))?;
    let mut command = Command::new(interpreter);
    command.arg(script).args(lines).stdin(Stdio::inherit()).stdout(Stdio::inherit()).stderr(Stdio::inherit());

    Ok(command.status()?.code().unwrap_or(2))
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(u8::try_from(code).unwrap_or(2)),
        Err(error) => { eprintln!("observer launcher: {error}"); ExitCode::from(2) }
    }
}
