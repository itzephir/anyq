use crate::{session_class::SessionClass, session_permit::SessionPermit};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

#[derive(Default)]
pub(crate) struct SessionCapacity {
    commands: AtomicUsize,
    waiting: AtomicUsize,
    controls: AtomicUsize,
}

impl SessionCapacity {
    pub fn admit(self: &Arc<Self>) -> Option<SessionPermit> {
        for class in [SessionClass::Command, SessionClass::Control] {
            if self.reserve(class) {
                return Some(SessionPermit {
                    capacity: self.clone(),
                    class,
                });
            }
        }

        None
    }

    fn counter(&self, class: SessionClass) -> (&AtomicUsize, usize) {
        match class {
            SessionClass::Command => (&self.commands, 64),
            SessionClass::Waiting => (&self.waiting, 64),
            SessionClass::Control => (&self.controls, 8),
        }
    }

    pub fn reserve(&self, class: SessionClass) -> bool {
        let (counter, limit) = self.counter(class);
        let mut used = counter.load(Ordering::SeqCst);

        while used < limit {
            match counter.compare_exchange_weak(used, used + 1, Ordering::SeqCst, Ordering::SeqCst)
            {
                Ok(_) => return true,
                Err(current) => used = current,
            }
        }

        false
    }

    pub fn release(&self, class: SessionClass) {
        self.counter(class).0.fetch_sub(1, Ordering::SeqCst);
    }
}
