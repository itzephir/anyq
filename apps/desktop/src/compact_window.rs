use crate::{compact_layout::CompactLayout, controller::Controller};
use std::sync::Arc;
use tauri::{Manager, PhysicalRect};

pub fn toggle(app: &tauri::AppHandle, position: tauri::PhysicalPosition<f64>, rect: tauri::Rect) {
    if cfg!(target_os = "linux") {
        return;
    }
    let Some(window) = app.get_webview_window("compact") else {
        return;
    };
    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
        return;
    }

    if place(&window, position, rect).is_err() {
        let _ = crate::commands::show_main(app.clone(), None, false);
        return;
    }
    let _ = window.show();
    let _ = window.set_focus();
    app.state::<Arc<Controller>>().refresh();
}

#[cfg(not(target_os = "macos"))]
fn place(
    window: &tauri::WebviewWindow,
    position: tauri::PhysicalPosition<f64>,
    rect: tauri::Rect,
) -> tauri::Result<()> {
    let monitor = window
        .monitor_from_point(position.x, position.y)?
        .ok_or(tauri::Error::InvalidWindowHandle)?;
    let scale = monitor.scale_factor();
    let layout = CompactLayout::anchored(
        *monitor.work_area(),
        PhysicalRect {
            position: rect.position.to_physical(scale),
            size: rect.size.to_physical(scale),
        },
        scale,
    );
    window.set_position(layout.position)?;
    window.set_size(layout.size)
}

#[cfg(target_os = "macos")]
fn place(
    window: &tauri::WebviewWindow,
    _position: tauri::PhysicalPosition<f64>,
    rect: tauri::Rect,
) -> tauri::Result<()> {
    use objc2::MainThreadMarker;
    use objc2_app_kit::{NSEvent, NSScreen};
    use tauri::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize};

    let marker = MainThreadMarker::new().ok_or(tauri::Error::InvalidWindowHandle)?;
    let screens = NSScreen::screens(marker);
    let primary = screens
        .firstObject()
        .ok_or(tauri::Error::InvalidWindowHandle)?;
    let cursor = NSEvent::mouseLocation();
    let screen = screens
        .iter()
        .find(|screen| {
            let frame = screen.frame();
            cursor.x >= frame.origin.x
                && cursor.x < frame.origin.x + frame.size.width
                && cursor.y >= frame.origin.y
                && cursor.y < frame.origin.y + frame.size.height
        })
        .ok_or(tauri::Error::InvalidWindowHandle)?;
    let scale = screen.backingScaleFactor();
    let bounds = screen.visibleFrame();
    let origin = rect.position.to_logical::<f64>(scale);
    let size = rect.size.to_logical::<f64>(scale);
    let layout = CompactLayout::anchored(
        PhysicalRect {
            position: PhysicalPosition::new(
                bounds.origin.x.round() as i32,
                (primary.frame().size.height - bounds.origin.y - bounds.size.height).round() as i32,
            ),
            size: PhysicalSize::new(bounds.size.width as u32, bounds.size.height as u32),
        },
        PhysicalRect {
            position: PhysicalPosition::new(origin.x, origin.y),
            size: PhysicalSize::new(size.width, size.height),
        },
        1.0,
    );
    window.set_position(LogicalPosition::new(layout.position.x, layout.position.y))?;
    window.set_size(LogicalSize::new(layout.size.width, layout.size.height))
}
