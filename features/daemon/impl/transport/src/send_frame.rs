use crate::{MORE, Stream};
use anyq_daemon_api::MAX_FRAME_BYTES;
use serde::Serialize;
use std::{
    io::{self, Write},
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant},
};

pub fn send_frame<T: Serialize>(
    stream: &Stream,
    value: &T,
    cancelled: &AtomicBool,
    timeout: Duration,
) -> io::Result<()> {
    send_bounded_frame(stream, value, cancelled, timeout, usize::MAX)
}

pub fn send_bounded_frame<T: Serialize>(
    stream: &Stream,
    value: &T,
    cancelled: &AtomicBool,
    timeout: Duration,
    max_bytes: usize,
) -> io::Result<()> {
    let payload = serde_json::to_vec(value).map_err(io::Error::other)?;
    if payload.len() > max_bytes {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "daemon response exceeds encoded size limit",
        ));
    }

    let mut idle_deadline = Instant::now() + timeout;
    let total_deadline = Instant::now() + Duration::from_secs(120);
    let mut offset = 0;

    while offset < payload.len() {
        let end = (offset + MAX_FRAME_BYTES).min(payload.len());
        let mut length = (end - offset) as u32;
        if end < payload.len() {
            length |= MORE;
        }

        write_piece(
            stream,
            &length.to_be_bytes(),
            cancelled,
            &mut idle_deadline,
            total_deadline,
            timeout,
        )?;
        write_piece(
            stream,
            &payload[offset..end],
            cancelled,
            &mut idle_deadline,
            total_deadline,
            timeout,
        )?;
        offset = end;
    }

    Ok(())
}

fn write_piece(
    stream: &Stream,
    buffer: &[u8],
    cancelled: &AtomicBool,
    idle_deadline: &mut Instant,
    total_deadline: Instant,
    timeout: Duration,
) -> io::Result<()> {
    let mut written = 0;
    let mut stream = stream;

    while written < buffer.len() {
        if cancelled.load(Ordering::SeqCst) {
            return Err(io::Error::new(io::ErrorKind::Interrupted, "interrupted"));
        }

        if Instant::now() >= total_deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "daemon transfer exceeded total time limit",
            ));
        }

        let end = (written + 64 * 1024).min(buffer.len());
        match stream.write(&buffer[written..end]) {
            Ok(0) => {
                return Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "daemon connection closed",
                ));
            }
            Ok(count) => {
                written += count;
                *idle_deadline = Instant::now() + timeout;
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                if Instant::now() >= *idle_deadline {
                    return Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        "daemon send timed out",
                    ));
                }

                thread::sleep(Duration::from_millis(2));
            }
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) => return Err(error),
        }
    }

    Ok(())
}
