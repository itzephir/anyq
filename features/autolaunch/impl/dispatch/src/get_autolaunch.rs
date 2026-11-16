use crate::ReadRegistration;
use anyq_autolaunch_api::GetAutolaunch;
use anyq_queue_api::Result;

pub struct GetAutolaunchInteractor<S> {
    registration: S,
}

impl<S> GetAutolaunchInteractor<S> {
    pub fn new(registration: S) -> Self {
        Self { registration }
    }
}

impl<S: ReadRegistration> GetAutolaunch for GetAutolaunchInteractor<S> {
    fn get(&mut self) -> Result<bool> {
        self.registration.registered()
    }
}
