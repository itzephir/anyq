use crate::{session_capacity::SessionCapacity, session_class::SessionClass};
use std::sync::Arc;

pub(crate) struct SessionPermit {
    pub capacity: Arc<SessionCapacity>,
    pub class: SessionClass,
}

impl SessionPermit {
    pub fn enter(&mut self, class: SessionClass) -> bool {
        if self.class == class {
            return true;
        }

        if !self.capacity.reserve(class) {
            return false;
        }

        self.capacity.release(self.class);
        self.class = class;

        true
    }
}

impl Drop for SessionPermit {
    fn drop(&mut self) {
        self.capacity.release(self.class);
    }
}
