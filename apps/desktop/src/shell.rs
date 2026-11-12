use crate::{commands, controller::Controller};
use anyq_desktop_api::Language;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tauri::{
    Manager,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};

pub struct Shell {
    pub usable: AtomicBool,
    status: MenuItem<tauri::Wry>,
    open: MenuItem<tauri::Wry>,
    refresh: MenuItem<tauri::Wry>,
    settings: MenuItem<tauri::Wry>,
    quit: MenuItem<tauri::Wry>,
}

pub fn host_available() -> bool {
    #[cfg(target_os = "linux")]
    {
        let host = || -> Result<bool, zbus::Error> {
            let connection = zbus::blocking::connection::Builder::session()?
                .method_timeout(std::time::Duration::from_secs(2))
                .build()?;
            let proxy = zbus::blocking::Proxy::new(
                &connection,
                "org.kde.StatusNotifierWatcher",
                "/StatusNotifierWatcher",
                "org.kde.StatusNotifierWatcher",
            )?;
            proxy.get_property::<bool>("IsStatusNotifierHostRegistered")
        };
        host().unwrap_or(false)
    }
    #[cfg(not(target_os = "linux"))]
    {
        true
    }
}

impl Shell {
    pub fn install(app: &tauri::AppHandle) -> tauri::Result<Self> {
        let status = MenuItem::with_id(app, "status", "Connecting…", false, None::<&str>)?;
        let open = MenuItem::with_id(app, "open", "Open window", true, None::<&str>)?;
        let refresh = MenuItem::with_id(app, "refresh", "Refresh", true, None::<&str>)?;
        let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
        let quit = MenuItem::with_id(app, "quit", "Quit anyq", true, None::<&str>)?;
        let menu = Menu::with_items(app, &[&status, &open, &refresh, &settings, &quit])?;
        TrayIconBuilder::with_id("queue")
            .icon(tauri::image::Image::from_bytes(include_bytes!(
                "../icons/tray/24x24.png"
            ))?)
            .icon_as_template(cfg!(target_os = "macos"))
            .tooltip("anyq")
            .menu(&menu)
            .show_menu_on_left_click(cfg!(target_os = "linux"))
            .on_menu_event(|app, event| match event.id.as_ref() {
                "open" => {
                    let _ = commands::show_main(app.clone(), None, false);
                }
                "settings" => {
                    let _ = commands::show_main(app.clone(), None, true);
                }
                "refresh" => app.state::<Arc<Controller>>().refresh(),
                "quit" => {
                    app.state::<Arc<Controller>>().stop();
                    app.exit(0);
                }
                _ => {}
            })
            .on_tray_icon_event(|tray, event| {
                if let TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    position,
                    rect,
                    ..
                } = event
                {
                    crate::compact_window::toggle(tray.app_handle(), position, rect);
                }
            })
            .build(app)?;

        Ok(Self {
            usable: AtomicBool::new(host_available()),
            status,
            open,
            refresh,
            settings,
            quit,
        })
    }

    pub fn update(&self, controller: &Controller) {
        let session = controller.session();
        let text: serde_json::Value =
            serde_json::from_str(if session.preferences.language == Language::Russian {
                include_str!("../ui/locales/native.ru.json")
            } else {
                include_str!("../ui/locales/native.en.json")
            })
            .expect("bundled localization");
        let summary = if session.model.status == anyq_desktop_api::ConnectionState::Live {
            session
                .model
                .snapshot
                .as_ref()
                .map(|snapshot| {
                    format!(
                        "{} {} · {} {}",
                        snapshot.count(anyq_queue_api::TaskState::Running),
                        text["running"].as_str().unwrap_or_default(),
                        snapshot.count(anyq_queue_api::TaskState::Queued),
                        text["queued"].as_str().unwrap_or_default()
                    )
                })
                .unwrap_or_default()
        } else {
            text[session.model.status.as_str()]
                .as_str()
                .unwrap_or("Connecting…")
                .to_owned()
        };
        drop(session);
        for (item, key) in [
            (&self.open, "open"),
            (&self.refresh, "refresh"),
            (&self.settings, "settings"),
            (&self.quit, "quit"),
        ] {
            let _ = item.set_text(text[key].as_str().unwrap_or(key));
        }
        let _ = self.status.set_text(summary);
    }

    pub fn refresh_host(&self, app: &tauri::AppHandle) {
        let available = host_available()
            && app.tray_by_id("queue").is_some_and(|tray| {
                cfg!(target_os = "linux") || tray.rect().ok().flatten().is_some()
            });
        let was_available = self.usable.swap(available, Ordering::SeqCst);
        if was_available && !available {
            // Restore a reachable ordinary window when a Linux status host disappears.
            let _ = commands::show_main(app.clone(), None, false);
        }
    }
}
