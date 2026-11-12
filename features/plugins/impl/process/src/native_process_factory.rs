use crate::{io_runtime::runtime, process_handle::ProcessHandle, supervise::supervise};
use anyq_plugin_api::PluginRegistration;
use anyq_plugin_runtime::{PluginProcess, ProcessFactory};
use anyq_queue_api::Result;
use std::sync::{Arc, atomic::AtomicBool};
use tokio::sync::mpsc;

pub struct NativeProcessFactory;

impl ProcessFactory for NativeProcessFactory {
    fn spawn(&self, registration: &PluginRegistration) -> Result<Box<dyn PluginProcess>> {
        let (outgoing, receiver) = mpsc::channel(32);
        let (sender, incoming) = mpsc::channel(8);
        let cancelled = Arc::new(AtomicBool::new(false));
        let terminated = Arc::new(AtomicBool::new(false));
        runtime().spawn(supervise(
            registration.clone(),
            receiver,
            sender,
            cancelled.clone(),
            terminated.clone(),
        ));

        Ok(Box::new(ProcessHandle {
            outgoing,
            incoming,
            cancelled,
            terminated,
        }))
    }
}
