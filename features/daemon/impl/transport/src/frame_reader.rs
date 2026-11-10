use crate::{MORE, Stream};
use anyq_daemon_api::MAX_FRAME_BYTES;
use serde::de::DeserializeOwned;
use std::{
    io::{self, Read},
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant},
};

pub struct FrameReader {
    prefix: [u8; 4],
    prefix_read: usize,
    payload: Vec<u8>,
    payload_read: usize,
    chunk_length: Option<usize>,
    more: bool,
    limit: usize,
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

impl Default for FrameReader {
    fn default() -> Self {
        Self::with_limit(MAX_FRAME_BYTES)
    }
}

impl FrameReader {
    pub(crate) fn with_limit(limit: usize) -> Self {
        Self {
            prefix: [0; 4],
            prefix_read: 0,
            payload: Vec::new(),
            payload_read: 0,
            chunk_length: None,
            more: false,
            limit,
        }
    }

    /// Queue query results can span bounded chunks without truncating large lists.
    /// Incoming requests retain the default whole-message limit.
    pub fn for_responses() -> Self {
        Self::with_limit(usize::MAX)
    }

    pub fn receive<T: DeserializeOwned>(
        &mut self,
        stream: &Stream,
        cancelled: &AtomicBool,
        timeout: Duration,
    ) -> io::Result<T> {
        let deadline = Instant::now() + timeout;

        loop {
            fill(
                stream,
                &mut self.prefix,
                &mut self.prefix_read,
                cancelled,
                deadline,
            )?;

            if self.chunk_length.is_none() {
                let header = u32::from_be_bytes(self.prefix);
                let length = (header & !MORE) as usize;
                if length == 0 || length > MAX_FRAME_BYTES {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "invalid daemon frame length",
                    ));
                }

                let total = self
                    .payload
                    .len()
                    .checked_add(length)
                    .filter(|total| *total <= self.limit)
                    .ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            "daemon request exceeds size limit",
                        )
                    })?;
                self.payload.resize(total, 0);
                self.chunk_length = Some(length);
                self.more = header & MORE != 0;
            }

            fill(
                stream,
                &mut self.payload,
                &mut self.payload_read,
                cancelled,
                deadline,
            )?;
            if self.more {
                self.prefix = [0; 4];
                self.prefix_read = 0;
                self.chunk_length = None;

                continue;
            }

            let value = serde_json::from_slice(&self.payload)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
            *self = Self::with_limit(self.limit);

            return Ok(value);
        }
    }
}
