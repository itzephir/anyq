use crate::ports::PreferenceStore;
use anyq_desktop_api::{GetPanelPreferences, PanelPreferences, Result};

pub struct GetPanelPreferencesInteractor<S> {
    store: S,
}

impl<S> GetPanelPreferencesInteractor<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S: PreferenceStore> GetPanelPreferences for GetPanelPreferencesInteractor<S> {
    fn get(&mut self) -> Result<PanelPreferences> {
        self.store.load()
    }
}
