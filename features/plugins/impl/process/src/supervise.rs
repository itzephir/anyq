use anyq_plugin_api::PluginRegistration;
use anyq_queue_api::{Error, Result};
use std::{
    process::Stdio,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    process::Command,
    sync::mpsc::{Receiver, Sender},
};

const MAX_FRAME: usize = 16 * 1024 * 1024;

async fn read_frames(
    stdout: tokio::process::ChildStdout,
    incoming: &Sender<Result<Vec<u8>>>,
) -> Result<()> {
    let mut reader = BufReader::new(stdout);
    loop {
        let mut frame = Vec::new();
        loop {
            let bytes = reader
                .fill_buf()
                .await
                .map_err(|_| Error::Storage("plugin protocol read failed".into()))?;
            if bytes.is_empty() {
                return Err(Error::Storage("plugin protocol closed".into()));
            }
            let end = bytes.iter().position(|byte| *byte == b'\n');
            let count = end.map_or(bytes.len(), |index| index + 1);
            if frame.len() + count > MAX_FRAME + 1 {
                return Err(Error::Invalid("plugin protocol limit exceeded".into()));
            }
            frame.extend_from_slice(&bytes[..count]);
            reader.consume(count);
            if end.is_some() {
                frame.pop();
                break;
            }
        }
        incoming
            .try_send(Ok(frame))
            .map_err(|_| Error::Invalid("plugin protocol rate limit exceeded".into()))?;
    }
}

async fn write_frames(
    mut stdin: tokio::process::ChildStdin,
    outgoing: &mut Receiver<Vec<u8>>,
) -> Result<()> {
    while let Some(frame) = outgoing.recv().await {
        if frame.len() > MAX_FRAME {
            return Err(Error::Invalid("plugin frame limit exceeded".into()));
        }
        stdin
            .write_all(&frame)
            .await
            .map_err(|_| Error::Storage("plugin protocol write failed".into()))?;
        stdin
            .write_all(b"\n")
            .await
            .map_err(|_| Error::Storage("plugin protocol write failed".into()))?;
        stdin
            .flush()
            .await
            .map_err(|_| Error::Storage("plugin protocol write failed".into()))?;
    }

    Ok(())
}

async fn diagnostics(mut stderr: tokio::process::ChildStderr) -> Result<()> {
    let (mut buffer, mut total) = ([0_u8; 4096], 0_usize);
    loop {
        let count = stderr
            .read(&mut buffer)
            .await
            .map_err(|_| Error::Storage("plugin diagnostics read failed".into()))?;
        if count == 0 {
            std::future::pending::<()>().await;
        }
        total += count;
        if total > 64 * 1024 {
            return Err(Error::Invalid("plugin diagnostic limit exceeded".into()));
        }
    }
}

pub(crate) async fn supervise(
    registration: PluginRegistration,
    mut outgoing: Receiver<Vec<u8>>,
    incoming: Sender<Result<Vec<u8>>>,
    cancelled: Arc<AtomicBool>,
    terminated: Arc<AtomicBool>,
) {
    let result = async {
        let mut command = Command::new(&registration.executable);
        command
            .args(&registration.arguments)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        #[cfg(unix)]
        command.process_group(0);

        #[cfg(windows)]
        {
            const DETACHED_PROCESS: u32 = 0x00000008;
            const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;

            command.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
        }

        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(_) => {
                terminated.store(true, Ordering::SeqCst);

                return Err(Error::Storage("plugin executable could not start".into()));
            }
        };
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| Error::Storage("plugin stdin unavailable".into()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| Error::Storage("plugin stdout unavailable".into()))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| Error::Storage("plugin stderr unavailable".into()))?;
        let cancellation = async {
            while !cancelled.load(Ordering::SeqCst) {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        };

        let result = tokio::select! {
            result = read_frames(stdout, &incoming) => result,
            result = write_frames(stdin, &mut outgoing) => result,
            result = diagnostics(stderr) => result,
            _ = child.wait() => Err(Error::Storage("plugin exited".into())),
            _ = cancellation => Ok(()),
        };
        let _ = child.start_kill();
        if !matches!(
            tokio::time::timeout(Duration::from_secs(1), child.wait()).await,
            Ok(Ok(_))
        ) {
            return Err(Error::Storage("plugin process cleanup is uncertain".into()));
        }
        terminated.store(true, Ordering::SeqCst);

        result
    }
    .await;
    if let Err(error) = result {
        let _ = incoming.try_send(Err(error));
    }
}
