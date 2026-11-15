use crate::{io_runtime::io_runtime, io_wake::poll_io};
use interprocess::local_socket::{PeerCreds, tokio::Stream as AsyncStream, traits::StreamCommon};
use std::{
    io::{self, Read, Write},
    pin::Pin,
};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

pub struct Stream {
    pub(crate) inner: AsyncStream,
}

impl Stream {
    pub fn peer_creds(&self) -> io::Result<PeerCreds> {
        self.inner.peer_creds()
    }
}

impl Read for &Stream {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let _runtime = io_runtime()?.enter();
        let mut source = &self.inner;
        let mut buffer = ReadBuf::new(buffer);

        poll_io(|context| Pin::new(&mut source).poll_read(context, &mut buffer))?;

        Ok(buffer.filled().len())
    }
}

impl Read for Stream {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        (&*self).read(buffer)
    }
}

impl Write for &Stream {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let _runtime = io_runtime()?.enter();
        let mut target = &self.inner;

        poll_io(|context| Pin::new(&mut target).poll_write(context, buffer))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Write for Stream {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        (&*self).write(buffer)
    }

    fn flush(&mut self) -> io::Result<()> {
        (&*self).flush()
    }
}
