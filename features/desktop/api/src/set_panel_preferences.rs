use crate::PanelPreferences;
use anyq_queue_api::Result;

pub trait SetPanelPreferences: Send {
    fn set(&mut self, preferences: PanelPreferences) -> Result<PanelPreferences>;
}
