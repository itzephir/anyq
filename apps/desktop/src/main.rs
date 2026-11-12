#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;
mod controller;
mod panel_session;
mod preference_input;
mod presentation;
mod shell;

#[cfg(feature = "integration-test-hooks")]
mod test_driver;

use anyq_desktop_api::PanelPreferences;
use anyq_queue_api::{Error, ObserveQueueRequest};
use controller::{Controller, DisplayClock};
use std::{
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use tauri::{Emitter, Manager};

fn main() {
    // Owners launched by the shared client never initialize a GUI or open a new queue.
    let arguments: Vec<_> = std::env::args_os().skip(1).collect();
    if arguments.len() == 4
        && arguments[0] == "daemon"
        && arguments[1] == "run"
        && arguments[2] == "--db"
    {
        let cancelled = Arc::new(AtomicBool::new(false));
        let signal = cancelled.clone();
        if ctrlc::set_handler(move || {
            signal.store(true, Ordering::SeqCst);
        })
        .is_err()
        {
            std::process::exit(2);
        }
        if anyq_daemon_impl::run(Path::new(&arguments[3]), cancelled).is_err() {
            std::process::exit(2);
        }
        return;
    }

    let result = run();
    if let Err(error) = result {
        eprintln!("anyq panel: {error}");
        std::process::exit(2);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let context = tauri::generate_context!();
    #[cfg(feature = "integration-test-hooks")]
    let context = test_driver::isolate_renderer(context);

    let config_path = anyq_config_impl::paths::home()?.join(".anyq/config.toml");
    let preference_path = anyq_config_impl::paths::home()?.join(".anyq/panel.toml");
    let preferences = anyq_desktop_impl::get_preferences(&preference_path).get();
    let resolve = Arc::new(move |preferences: &PanelPreferences| {
        let config = anyq_config_impl::open(&config_path, Arc::new(AtomicBool::new(false)))
            .get_config
            .get()
            .map_err(|error| Error::Storage(error.to_string()))?;
        let mut selection = anyq_config_impl::paths::resolve_database(
            preferences.database_override.as_deref().map(Path::new),
            config.db_path(),
        )
        .map_err(|error| Error::Storage(error.to_string()))?;
        let absolute = std::path::absolute(&selection.path)
            .map_err(|error| Error::Storage(error.to_string()))?;
        selection.path = anyq_daemon_impl::canonical_database_path(&absolute).unwrap_or(absolute);

        Ok(selection)
    });
    let observe = Arc::new(|path: &Path, cancelled: Arc<AtomicBool>| {
        anyq_daemon_impl::observe(path, cancelled)?.observe(ObserveQueueRequest::default())
    });
    let clock: Arc<DisplayClock> = Arc::new(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|value| value.as_millis() as u64)
            .unwrap_or_default()
    });
    #[cfg(feature = "integration-test-hooks")]
    let clock: Arc<DisplayClock> = match test_driver::fixed_clock()? {
        Some(timestamp) => Arc::new(move || timestamp),
        None => clock,
    };
    let controller = Arc::new(Controller::new(
        preferences,
        anyq_desktop_impl::set_preferences(&preference_path),
        resolve,
        observe,
        clock,
    ));
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(controller.clone());
    #[cfg(feature = "integration-test-hooks")]
    let builder = builder.plugin(test_driver::clock_plugin());
    let app = builder
        .invoke_handler(tauri::generate_handler![
            commands::get_panel_state,
            commands::refresh_panel,
            commands::save_preferences,
            commands::choose_database,
            commands::show_main,
            commands::dismiss_compact,
            #[cfg(feature = "integration-test-hooks")]
            test_driver::test_report
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            #[cfg(feature = "integration-test-hooks")]
            test_driver::install(&handle)?;
            if let Ok(shell) = shell::Shell::install(&handle) {
                app.manage(shell);
            }
            let controller = app.state::<Arc<Controller>>().inner().clone();
            std::thread::spawn(move || {
                controller.run(|| {
                    let _ = handle.emit("panel-state", presentation::state(&controller));
                    if let Some(shell) = handle.try_state::<shell::Shell>() {
                        shell.update(&controller);
                        shell.refresh_host(&handle);
                    }
                    let visible = ["main", "compact"].into_iter().any(|label| {
                        handle
                            .get_webview_window(label)
                            .is_some_and(|window| window.is_visible().unwrap_or(false))
                    });
                    controller.set_visible(visible);
                });
            });
            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } if window.label() == "main" => {
                let app = window.app_handle();
                if app
                    .try_state::<shell::Shell>()
                    .is_some_and(|shell| shell.usable.load(Ordering::SeqCst))
                {
                    api.prevent_close();
                    let _ = window.hide();
                } else {
                    app.state::<Arc<Controller>>().stop();
                    app.exit(0);
                }
            }
            tauri::WindowEvent::Focused(false) if window.label() == "compact" => {
                let _ = window.hide();
            }
            tauri::WindowEvent::Focused(true) => window.state::<Arc<Controller>>().refresh(),
            _ => {}
        })
        .build(context)?;
    app.run(move |_app, event| match event {
        tauri::RunEvent::Exit | tauri::RunEvent::ExitRequested { .. } => controller.stop(),
        #[cfg(target_os = "macos")]
        tauri::RunEvent::Reopen { .. } => {
            let _ = commands::show_main(_app.clone(), None, false);
        }
        _ => {}
    });

    Ok(())
}

#[cfg(test)]
mod tests;
