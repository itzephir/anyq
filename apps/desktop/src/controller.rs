use crate::panel_session::PanelSession;
use anyq_config_api::DatabaseSelection;
use anyq_desktop_api::{PanelModel, PanelPreferences, SetPanelPreferences};
use anyq_queue_api::{QueueSnapshot, Result};
use std::{
    path::Path,
    sync::{
        Arc, Condvar, Mutex, MutexGuard,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

pub type ResolveDatabase = dyn Fn(&PanelPreferences) -> Result<DatabaseSelection> + Send + Sync;
pub type ObserveSnapshot = dyn Fn(&Path, Arc<AtomicBool>) -> Result<QueueSnapshot> + Send + Sync;

pub struct Controller {
    session: Mutex<PanelSession>,
    wake: Condvar,
    writer: Mutex<Box<dyn SetPanelPreferences>>,
    resolve: Arc<ResolveDatabase>,
    observe: Arc<ObserveSnapshot>,
}

impl Controller {
    pub fn new(
        preferences: Result<PanelPreferences>,
        writer: Box<dyn SetPanelPreferences>,
        resolve: Arc<ResolveDatabase>,
        observe: Arc<ObserveSnapshot>,
    ) -> Self {
        let (preferences, preference_error) = match preferences {
            Ok(preferences) => (preferences, None),
            Err(error) => (PanelPreferences::default(), Some(error.to_string())),
        };
        let mut model = PanelModel::default();
        model.change_source();

        Self {
            session: Mutex::new(PanelSession {
                model,
                preferences,
                selection: None,
                preference_error,
                last_success: None,
                active_cancel: None,
                refresh: true,
                next_poll: Instant::now(),
                visible: true,
                stopped: false,
            }),
            wake: Condvar::new(),
            writer: Mutex::new(writer),
            resolve,
            observe,
        }
    }

    pub fn session(&self) -> MutexGuard<'_, PanelSession> {
        self.session
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub fn refresh(&self) {
        self.session().refresh = true;
        self.wake.notify_one();
    }

    pub fn set_visible(&self, visible: bool) {
        let mut session = self.session();
        if visible && !session.visible {
            session.refresh = true;
        }
        session.visible = visible;
        self.wake.notify_one();
    }

    pub fn save(&self, preferences: PanelPreferences) -> Result<()> {
        preferences.validate()?;
        let mut writer = self
            .writer
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // Persist before changing active state; even an unavailable target is an explicit selection.
        writer.set(preferences.clone())?;
        let mut session = self.session();
        let changed = session.preferences.database_override != preferences.database_override
            || session.preference_error.is_some();
        session.preferences = preferences;
        session.preference_error = None;
        if changed {
            if let Some(cancelled) = &session.active_cancel {
                cancelled.store(true, Ordering::SeqCst);
            }
            session.model.change_source();
            session.selection = None;
            session.last_success = None;
        }
        session.refresh = true;
        self.wake.notify_one();

        Ok(())
    }

    pub fn stop(&self) {
        let mut session = self.session();
        session.stopped = true;
        if let Some(cancelled) = &session.active_cancel {
            cancelled.store(true, Ordering::SeqCst);
        }
        self.wake.notify_one();
    }

    pub fn run(&self, publish: impl Fn()) {
        loop {
            let mut session = self.session();
            while !session.stopped && !session.refresh && Instant::now() < session.next_poll {
                let remaining = session.next_poll.saturating_duration_since(Instant::now());
                session = self
                    .wake
                    .wait_timeout(session, remaining)
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .0;
            }
            if session.stopped {
                break;
            }
            session.refresh = false;
            let generation = session.model.generation;
            let preferences = session.preferences.clone();
            let preference_error = session.preference_error.clone();
            let cancelled = Arc::new(AtomicBool::new(false));
            session.active_cancel = Some(cancelled.clone());
            drop(session);

            // One worker and one bounded watchdog: refresh clicks coalesce, never overlap.
            let (done, completion) = std::sync::mpsc::channel::<()>();
            let deadline_cancel = cancelled.clone();
            let watchdog = std::thread::spawn(move || {
                if completion.recv_timeout(Duration::from_secs(10)).is_err() {
                    deadline_cancel.store(true, Ordering::SeqCst);
                }
            });
            let result = if let Some(error) = preference_error {
                Err(anyq_queue_api::Error::Storage(error))
            } else {
                (self.resolve)(&preferences).and_then(|selection| {
                    let mut session = self.session();
                    if generation != session.model.generation || session.stopped {
                        return Err(anyq_queue_api::Error::Interrupted);
                    }
                    if session
                        .selection
                        .as_ref()
                        .is_some_and(|old| old != &selection)
                    {
                        // Automatic config changes are source changes too.
                        session.model.change_source();
                        session.selection = Some(selection.clone());
                        session.last_success = None;
                        session.refresh = true;
                        return Err(anyq_queue_api::Error::Interrupted);
                    }
                    session.selection = Some(selection.clone());
                    drop(session);
                    (self.observe)(&selection.path, cancelled.clone())
                })
            };
            let _ = done.send(());
            let _ = watchdog.join();

            let mut session = self.session();
            session.active_cancel = None;
            match result {
                Ok(snapshot) => {
                    if session.model.accept(generation, snapshot) {
                        session.last_success = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .ok()
                            .map(|time| time.as_millis() as u64);
                    }
                }
                Err(error) => {
                    session.model.fail(generation, error.to_string());
                }
            }
            session.next_poll =
                Instant::now() + Duration::from_secs(if session.visible { 1 } else { 5 });
            drop(session);
            publish();
        }
    }
}
