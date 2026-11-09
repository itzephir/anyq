use crate::{checkpoint::checkpoint, error::Result};
use std::{
    io::{self, Write},
    sync::{atomic::AtomicBool, mpsc},
    thread,
    time::Duration,
};

pub fn emit(message: &str, cancelled: &AtomicBool) -> Result<()> {
    let message = format!("{message}\n");

    write(
        move || io::stdout().lock().write_all(message.as_bytes()),
        cancelled,
    )
}

pub fn emit_error(message: &str, cancelled: &AtomicBool) -> Result<()> {
    let message = format!("{message}\n");

    write(
        move || io::stderr().lock().write_all(message.as_bytes()),
        cancelled,
    )
}

pub fn write(
    operation: impl FnOnce() -> io::Result<()> + Send + 'static,
    cancelled: &AtomicBool,
) -> Result<()> {
    checkpoint("cli.before-output", cancelled)?;
    let (completed, result) = mpsc::sync_channel(1);
    // The writer owns no domain state; cancellation unwinds the main thread without joining it.
    thread::Builder::new()
        .name("anyq-output".into())
        .spawn(move || {
            let _ = completed.send(operation());
        })?;

    loop {
        match result.recv_timeout(Duration::from_millis(20)) {
            Ok(result) => {
                checkpoint("cli.after-output", cancelled)?;
                return result.map_err(Into::into);
            }
            Err(mpsc::RecvTimeoutError::Timeout) => checkpoint("cli.output-blocked", cancelled)?,
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return Err(io::Error::other("output writer stopped before completion").into());
            }
        }
    }
}
