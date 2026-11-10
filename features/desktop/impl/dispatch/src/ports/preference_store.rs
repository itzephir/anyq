use anyq_desktop_api::{PanelPreferences, Result};

pub trait PreferenceStore: Send {
    fn load(&mut self) -> Result<PanelPreferences>;

    fn save(&mut self, preferences: &PanelPreferences) -> Result<()>;
}
