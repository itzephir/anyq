use crate::{ReadRegistration, WriteRegistration};
use anyq_autolaunch_api::{SetAutolaunch, SetAutolaunchRequest};
use anyq_queue_api::{Error, Result};

pub struct SetAutolaunchInteractor<S> {
    registration: S,
}

impl<S> SetAutolaunchInteractor<S> {
    pub fn new(registration: S) -> Self {
        Self { registration }
    }
}

impl<S: ReadRegistration + WriteRegistration> SetAutolaunch for SetAutolaunchInteractor<S> {
    fn set(&mut self, request: SetAutolaunchRequest) -> Result<bool> {
        if request.enabled {
            self.registration.register()?;
        } else {
            self.registration.unregister()?;
        }
        let enabled = self.registration.registered()?;
        if enabled != request.enabled {
            return Err(Error::Conflict(
                "system autolaunch registration changed concurrently".into(),
            ));
        }

        Ok(enabled)
    }
}
