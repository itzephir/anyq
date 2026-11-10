use anyq_config_api::DatabaseSelection;
use anyq_desktop_api::{PanelModel, PanelPreferences};
use std::sync::{Arc, atomic::AtomicBool};
use std::time::Instant;

pub struct PanelSession {
    pub model: PanelModel,
    pub revision: u64,
    pub preferences: PanelPreferences,
    pub selection: Option<DatabaseSelection>,
    pub preference_error: Option<String>,
    pub last_success: Option<u64>,
    pub active_cancel: Option<Arc<AtomicBool>>,
    pub refresh: bool,
    pub next_poll: Instant,
    pub visible: bool,
    pub stopped: bool,
}
