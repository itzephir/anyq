//! Explicit desktop composition root.
use anyq_desktop_api::{GetPanelPreferences, SetPanelPreferences};
use anyq_desktop_dispatch::{GetPanelPreferencesInteractor, SetPanelPreferencesInteractor};
use anyq_desktop_file::FilePreferences;
use std::path::Path;

pub fn get_preferences(path: &Path) -> Box<dyn GetPanelPreferences> {
    Box::new(GetPanelPreferencesInteractor::new(FilePreferences::new(
        path,
    )))
}

pub fn set_preferences(path: &Path) -> Box<dyn SetPanelPreferences> {
    Box::new(SetPanelPreferencesInteractor::new(FilePreferences::new(
        path,
    )))
}
