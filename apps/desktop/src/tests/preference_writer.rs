use anyq_desktop_api::{PanelPreferences, SetPanelPreferences};
use anyq_queue_api::{Error, Result};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

pub struct PreferenceWriter(pub Arc<AtomicBool>);

impl SetPanelPreferences for PreferenceWriter {
    fn set(&mut self, preferences: PanelPreferences) -> Result<PanelPreferences> {
        if self.0.load(Ordering::SeqCst) {
            return Err(Error::Storage("write denied".into()));
        }

        Ok(preferences)
    }
}
