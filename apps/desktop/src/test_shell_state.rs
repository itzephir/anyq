use serde_json::{Value, json};
use std::sync::atomic::Ordering;
use tauri::Manager;

pub fn snapshot(app: &tauri::AppHandle) -> tauri::Result<Value> {
    let windows = ["main", "compact"]
        .into_iter()
        .map(|label| {
            let window = app
                .get_webview_window(label)
                .ok_or(tauri::Error::InvalidWindowHandle)?;
            Ok((
                label.to_owned(),
                json!({
                    "visible": window.is_visible()?,
                    "focused": window.is_focused()?,
                    "position": window.outer_position().ok(),
                    "size": window.outer_size()?,
                    "scale": window.scale_factor()?,
                    "monitor": window.current_monitor()?.map(|monitor| json!({"position":monitor.position(), "size":monitor.size(), "scale":monitor.scale_factor()}))
                }),
            ))
        })
        .collect::<tauri::Result<serde_json::Map<_, _>>>()?;
    let monitors = app
        .available_monitors()?
        .into_iter()
        .map(|monitor| {
            json!({
                "name": monitor.name(),
                "position": monitor.position(),
                "size": monitor.size(),
                "work_area": monitor.work_area(),
                "scale": monitor.scale_factor()
            })
        })
        .collect::<Vec<_>>();
    let tray = app.tray_by_id("queue");
    let rect = tray.as_ref().map(|tray| tray.rect()).transpose()?.flatten();
    let usable = app
        .try_state::<crate::shell::Shell>()
        .is_some_and(|shell| shell.usable.load(Ordering::SeqCst));

    Ok(json!({"windows": windows, "monitors": monitors, "tray": {"usable": usable, "rect":rect}}))
}
