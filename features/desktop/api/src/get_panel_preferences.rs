use crate::PanelPreferences;
use anyq_queue_api::Result;

pub trait GetPanelPreferences: Send {
    fn get(&mut self) -> Result<PanelPreferences>;
}
