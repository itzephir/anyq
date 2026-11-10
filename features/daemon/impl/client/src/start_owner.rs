use anyq_daemon_transport::{Endpoint, Stream};
use std::{
    io,
    process::{Command, Stdio},
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant},
};

pub(crate) fn start_owner(endpoint: &Endpoint, cancelled: &AtomicBool) -> io::Result<Stream> {
    if let Ok(stream) = endpoint.connect() {
        return Ok(stream);
    }

    let mut command = Command::new(std::env::current_exe()?);
    command
        .args(["daemon", "run", "--db"])
        .arg(&endpoint.database)
        .env("ANYQ_DAEMON_BACKGROUND", "1")
        .env_remove("ANYQ_TEST_CHECKPOINT")
        .env_remove("ANYQ_TEST_CHECKPOINT_HIT")
        .env_remove("ANYQ_TEST_CHECKPOINT_DIR")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;

        command.process_group(0);
    }

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;

        command.creation_flags(0x00000008 | 0x00000200);
    }

    let mut child = command.spawn()?;
    let deadline = Instant::now() + Duration::from_secs(10);

    loop {
        if cancelled.load(Ordering::SeqCst) {
            return Err(io::Error::new(io::ErrorKind::Interrupted, "interrupted"));
        }

        if let Ok(stream) = endpoint.connect() {
            let _ = child.try_wait();

            return Ok(stream);
        }

        let _ = child.try_wait();
        if Instant::now() >= deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "queue daemon did not become available",
            ));
        }

        thread::sleep(Duration::from_millis(5));
    }
}
