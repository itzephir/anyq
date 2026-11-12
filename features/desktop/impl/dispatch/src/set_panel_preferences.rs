use crate::ports::PreferenceStore;
use anyq_desktop_api::{PanelPreferences, Result, SetPanelPreferences};

pub struct SetPanelPreferencesInteractor<S> {
    store: S,
}

impl<S> SetPanelPreferencesInteractor<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S: PreferenceStore> SetPanelPreferences for SetPanelPreferencesInteractor<S> {
    fn set(&mut self, preferences: PanelPreferences) -> Result<PanelPreferences> {
        preferences.validate()?;
        self.store.save(&preferences)?;

        Ok(preferences)
    }
}
