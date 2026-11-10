use crate::io_runtime::io_runtime;
use interprocess::local_socket::{PeerCreds, tokio::Stream as AsyncStream, traits::StreamCommon};
use std::{
    io::{self, Read, Write},
    pin::Pin,
    task::{Context, Poll, Waker},
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
        let mut context = Context::from_waker(Waker::noop());
        let mut buffer = ReadBuf::new(buffer);

        match Pin::new(&mut source).poll_read(&mut context, &mut buffer) {
            Poll::Ready(Ok(())) => Ok(buffer.filled().len()),
            Poll::Ready(Err(error)) => Err(error),
            Poll::Pending => Err(io::ErrorKind::WouldBlock.into()),
        }
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
        let mut context = Context::from_waker(Waker::noop());

        match Pin::new(&mut target).poll_write(&mut context, buffer) {
            Poll::Ready(result) => result,
            Poll::Pending => Err(io::ErrorKind::WouldBlock.into()),
        }
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
