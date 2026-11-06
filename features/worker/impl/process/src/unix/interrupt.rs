use std::{
    io,
    sync::{Arc, atomic::AtomicBool},
};

pub struct Interrupt {
    registrations: Vec<signal_hook::SigId>,
}

impl Interrupt {
    pub fn install(cancelled: Arc<AtomicBool>) -> io::Result<Self> {
        let mut result = Self {
            registrations: Vec::new(),
        };

        for signal in [signal_hook::consts::SIGINT, signal_hook::consts::SIGTERM] {
            result
                .registrations
                .push(signal_hook::flag::register(signal, cancelled.clone())?);
        }

        Ok(result)
    }
}

impl Drop for Interrupt {
    fn drop(&mut self) {
        for id in self.registrations.drain(..) {
            signal_hook::low_level::unregister(id);
        }
    }
}
