use anyq_plugin_runtime::PluginProcess;
use anyq_queue_api::{Error, Result};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::sync::mpsc::{Receiver, Sender};

pub(crate) struct ProcessHandle {
    pub outgoing: Sender<Vec<u8>>,
    pub incoming: Receiver<Result<Vec<u8>>>,
    pub cancelled: Arc<AtomicBool>,
    pub terminated: Arc<AtomicBool>,
}

impl PluginProcess for ProcessHandle {
    fn send(&mut self, frame: Vec<u8>) -> Result<()> {
        self.outgoing
            .try_send(frame)
            .map_err(|_| Error::Storage("plugin output channel blocked".into()))
    }

    fn receive(&mut self) -> Result<Option<Vec<u8>>> {
        match self.incoming.try_recv() {
            Ok(result) => result.map(Some),
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => Ok(None),
            Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                Err(Error::Storage("plugin exited".into()))
            }
        }
    }

    fn stop(&mut self) -> Result<()> {
        self.cancelled.store(true, Ordering::SeqCst);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        while !self.terminated.load(Ordering::SeqCst) {
            if std::time::Instant::now() >= deadline {
                return Err(Error::Storage("plugin process cleanup is uncertain".into()));
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }

        Ok(())
    }
}

impl Drop for ProcessHandle {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }
}
