//! Typed desktop preference use cases and consumer-owned ports.
pub mod ports;

mod get_panel_preferences;
pub use get_panel_preferences::GetPanelPreferencesInteractor;

mod set_panel_preferences;
pub use set_panel_preferences::SetPanelPreferencesInteractor;
