use crate::Stream;
use crate::io_runtime::io_runtime;
use interprocess::local_socket::{tokio::Listener as AsyncListener, traits::tokio::Listener as _};
use std::{
    future::Future,
    io,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};

type AcceptFuture =
    Pin<Box<dyn Future<Output = io::Result<interprocess::local_socket::tokio::Stream>> + Send>>;

pub struct Listener {
    pub(crate) inner: Arc<AsyncListener>,
    pub(crate) pending: Mutex<Option<AcceptFuture>>,
}

impl Listener {
    pub fn accept(&self) -> io::Result<Stream> {
        let _runtime = io_runtime()?.enter();
        let mut pending = self
            .pending
            .lock()
            .map_err(|_| io::Error::other("daemon accept lock poisoned"))?;
        if pending.is_none() {
            let listener = self.inner.clone();
            *pending = Some(Box::pin(async move { listener.accept().await }));
        }

        let mut context = Context::from_waker(Waker::noop());
        match pending
            .as_mut()
            .expect("accept future initialized")
            .as_mut()
            .poll(&mut context)
        {
            Poll::Ready(result) => {
                *pending = None;

                result.map(|inner| Stream { inner })
            }
            Poll::Pending => Err(io::ErrorKind::WouldBlock.into()),
        }
    }
}
