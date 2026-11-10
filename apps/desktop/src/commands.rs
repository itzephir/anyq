use crate::{controller::Controller, preference_input::PreferenceInput, presentation};
use std::sync::Arc;
use tauri::{Emitter, Manager};
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
pub fn get_panel_state(controller: tauri::State<'_, Arc<Controller>>) -> serde_json::Value {
    presentation::state(&controller)
}

#[tauri::command]
pub fn refresh_panel(controller: tauri::State<'_, Arc<Controller>>) {
    controller.refresh();
}

#[tauri::command]
pub async fn save_preferences(
    app: tauri::AppHandle,
    controller: tauri::State<'_, Arc<Controller>>,
    preferences: PreferenceInput,
) -> Result<serde_json::Value, String> {
    let preferences = preferences.parse()?;
    let controller = controller.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        controller
            .save(preferences)
            .map_err(|error| error.to_string())?;
        let state = presentation::state(&controller);
        app.emit("panel-state", &state)
            .map_err(|error| error.to_string())?;

        Ok(state)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn choose_database(app: tauri::AppHandle) -> Option<String> {
    tauri::async_runtime::spawn_blocking(move || {
        app.dialog()
            .file()
            .blocking_pick_file()
            .and_then(|path| path.into_path().ok())
            .map(|path| path.to_string_lossy().into_owned())
    })
    .await
    .ok()
    .flatten()
}

#[tauri::command]
pub fn show_main(
    app: tauri::AppHandle,
    topic: Option<String>,
    settings: bool,
) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or("main window is unavailable")?;
    let _ = app.emit_to(
        "main",
        "panel-navigation",
        serde_json::json!({"topic": topic, "settings": settings}),
    );
    window.show().map_err(|error| error.to_string())?;
    window.unminimize().map_err(|error| error.to_string())?;
    window.set_focus().map_err(|error| error.to_string())?;
    if let Some(compact) = app.get_webview_window("compact") {
        let _ = compact.hide();
    }

    Ok(())
}
