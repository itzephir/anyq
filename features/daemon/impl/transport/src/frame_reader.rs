use anyq_daemon_api::MAX_FRAME_BYTES;
use interprocess::local_socket::Stream;
use serde::de::DeserializeOwned;
use std::{
    io::{self, Read},
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant},
};

#[derive(Default)]
pub struct FrameReader {
    prefix: [u8; 4],
    prefix_read: usize,
    payload: Vec<u8>,
    payload_read: usize,
}

fn fill(
    stream: &Stream,
    buffer: &mut [u8],
    read: &mut usize,
    cancelled: &AtomicBool,
    deadline: Instant,
) -> io::Result<()> {
    let mut stream = stream;

    while *read < buffer.len() {
        if cancelled.load(Ordering::SeqCst) {
            return Err(io::Error::new(io::ErrorKind::Interrupted, "interrupted"));
        }

        match stream.read(&mut buffer[*read..]) {
            Ok(0) => {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "daemon connection closed",
                ));
            }
            Ok(count) => *read += count,
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                if Instant::now() >= deadline {
                    return Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        "daemon receive timed out",
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

impl FrameReader {
    pub fn receive<T: DeserializeOwned>(
        &mut self,
        stream: &Stream,
        cancelled: &AtomicBool,
        timeout: Duration,
    ) -> io::Result<T> {
        let deadline = Instant::now() + timeout;
        fill(
            stream,
            &mut self.prefix,
            &mut self.prefix_read,
            cancelled,
            deadline,
        )?;
        let length = u32::from_be_bytes(self.prefix) as usize;

        if length == 0 || length > MAX_FRAME_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid daemon frame length",
            ));
        }

        if self.payload.is_empty() {
            self.payload.resize(length, 0);
        }

        fill(
            stream,
            &mut self.payload,
            &mut self.payload_read,
            cancelled,
            deadline,
        )?;
        let value = serde_json::from_slice(&self.payload)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        *self = Self::default();

        Ok(value)
    }
}
