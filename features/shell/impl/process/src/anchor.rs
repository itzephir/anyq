use anyq_queue_api::{Error, Result};

/// Private bundled process-group anchor. It never executes task payloads.
pub fn run() -> Result<()> {
    #[cfg(unix)]
    {
        use rustix::process::{
            Signal, getpgid, getpgrp, getpid, getppid, kill_process_group, setpgid,
        };
        use std::io::{Read, Write};
        let identity = getpid();
        if getpgrp() != identity {
            return Err(Error::Invalid(
                "shell anchor requires a private process group".into(),
            ));
        }
        let parent =
            getppid().ok_or_else(|| Error::Invalid("shell anchor has no parent".into()))?;
        let parent_group =
            getpgid(Some(parent)).map_err(|error| Error::Storage(error.to_string()))?;
        let mut input = std::io::stdin().lock();
        let mut output = std::io::stdout().lock();
        output
            .write_all(b"R")
            .and_then(|_| output.flush())
            .map_err(|error| Error::Storage(error.to_string()))?;
        let mut command = [0_u8];
        loop {
            match input.read_exact(&mut command) {
                Ok(()) if command[0] == b'Q' => return Ok(()),
                Ok(()) if command[0] == b'L' => {
                    // Keep this PID alive, but leave the target group so its
                    // disappearance can be tested atomically by the kernel.
                    setpgid(None, Some(parent_group))
                        .map_err(|error| Error::Storage(error.to_string()))?;
                    output
                        .write_all(b"R")
                        .and_then(|_| output.flush())
                        .map_err(|error| Error::Storage(error.to_string()))?;
                }
                _ => {
                    // EOF means the supervising plugin died. The still-owned
                    // anchor PID prevents reuse of the target group identifier.
                    let _ = kill_process_group(identity, Signal::KILL);
                    return Ok(());
                }
            }
        }
    }
    #[cfg(windows)]
    Err(Error::Invalid(
        "shell anchors are Unix-only; Windows uses native Jobs".into(),
    ))
}
