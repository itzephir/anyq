//! Desktop preferences and observation state. No platform or IO dependencies.
mod language;
pub use language::Language;

mod theme;
pub use theme::Theme;

mod panel_preferences;
pub use panel_preferences::PanelPreferences;

mod connection_state;
pub use connection_state::ConnectionState;

mod panel_model;
pub use panel_model::PanelModel;

mod get_panel_preferences;
pub use get_panel_preferences::GetPanelPreferences;

mod set_panel_preferences;
pub use set_panel_preferences::SetPanelPreferences;

pub use anyq_queue_api::{Error, Result};
