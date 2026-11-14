use anyq_daemon_transport::{Endpoint, Stream};
use std::{
    io,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant},
};

#[cfg(unix)]
use std::process::{Command, Stdio};

#[cfg(windows)]
use windows_spawn::{Command, CreationFlags, SpawnOptions, Stdio};

pub(crate) fn start_owner(
    endpoint: &Endpoint,
    cancelled: &AtomicBool,
    executable: Option<&Path>,
) -> io::Result<(Endpoint, Stream)> {
    let mut endpoint = Endpoint::resolve(&endpoint.database)?;
    if let Ok(stream) = endpoint.connect() {
        return Ok((endpoint, stream));
    }

    let default_executable;
    let executable = match executable {
        Some(executable) => executable,
        None => {
            default_executable = std::env::current_exe()?;
            &default_executable
        }
    };
    let mut command = Command::new(executable);
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

    #[cfg(unix)]
    let mut child = command.spawn()?;

    #[cfg(windows)]
    let mut child =
        command
            .spawn_with(SpawnOptions::new().creation_flags(
                CreationFlags::DETACHED_PROCESS | CreationFlags::NEW_PROCESS_GROUP,
            ))?;

    let deadline = Instant::now() + Duration::from_secs(10);

    loop {
        if cancelled.load(Ordering::SeqCst) {
            return Err(io::Error::new(io::ErrorKind::Interrupted, "interrupted"));
        }

        endpoint = Endpoint::resolve(&endpoint.database)?;
        if let Ok(stream) = endpoint.connect() {
            let _ = child.try_wait();

            return Ok((endpoint, stream));
        }

        if child.try_wait()?.is_some() && !anyq_daemon_transport::OwnerLock::is_held(&endpoint)? {
            #[cfg(unix)]
            {
                child = command.spawn()?;
            }

            #[cfg(windows)]
            {
                child = command.spawn_with(SpawnOptions::new().creation_flags(
                    CreationFlags::DETACHED_PROCESS | CreationFlags::NEW_PROCESS_GROUP,
                ))?;
            }
        }

        if Instant::now() >= deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "queue daemon did not become available",
            ));
        }

        thread::sleep(Duration::from_millis(5));
    }
}
