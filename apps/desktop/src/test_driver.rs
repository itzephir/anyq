//! Private, feature-gated native UI test transport; absent from shipping builds.
use crate::{commands, controller::Controller};
use serde_json::{Value, json};
use std::{fs, path::PathBuf, sync::Arc, time::Duration};
use tauri::{Emitter, Manager};

pub struct TestDriver {
    directory: PathBuf,
}

fn write_response(driver: &TestDriver, id: u64, value: Value) -> Result<(), String> {
    let temporary = driver.directory.join(format!("response-{id}.tmp"));
    let target = driver.directory.join(format!("response-{id}.json"));
    fs::write(
        &temporary,
        serde_json::to_vec(&value).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    fs::rename(temporary, target).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn test_report(
    window: tauri::WebviewWindow,
    driver: tauri::State<'_, TestDriver>,
    id: u64,
    payload: Value,
) -> Result<(), String> {
    let position = window.inner_position().map_err(|error| error.to_string())?;
    let size = window.inner_size().map_err(|error| error.to_string())?;
    let scale = window.scale_factor().map_err(|error| error.to_string())?;
    let tray = window
        .app_handle()
        .try_state::<crate::shell::Shell>()
        .is_some_and(|shell| shell.usable.load(std::sync::atomic::Ordering::SeqCst));

    let outer_position = window.outer_position().map_err(|error| error.to_string())?;
    let outer_size = window.outer_size().map_err(|error| error.to_string())?;
    #[cfg(windows)]
    let native_handle = window.hwnd().ok().map(|value| value.0 as usize);
    #[cfg(not(windows))]
    let native_handle: Option<usize> = None;
    write_response(
        &driver,
        id,
        json!({ "payload": payload, "rect": {"x":position.x,"y":position.y,"width":size.width,"height":size.height,"scale":scale}, "tray":tray, "native_handle":native_handle, "outer": {"x":outer_position.x,"y":outer_position.y,"width":outer_size.width,"height":outer_size.height} }),
    )
}

pub fn install(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let Some(directory) = std::env::var_os("ANYQ_PANEL_TEST_DIR").map(PathBuf::from) else {
        return Ok(());
    };
    if !directory.is_absolute() || !directory.is_dir() {
        return Err("test driver requires an existing absolute private directory".into());
    }
    app.manage(TestDriver { directory });
    let app = app.clone();
    std::thread::spawn(move || {
        let mut seen = 0;
        while !app.state::<Arc<Controller>>().session().stopped {
            let driver = app.state::<TestDriver>();
            // Each request has its own immutable file: Windows can keep a reader
            // open without blocking publication of the next request.
            let command = fs::read(driver.directory.join(format!("command-{}.json", seen + 1)))
                .ok()
                .filter(|bytes| bytes.len() <= 65_536)
                .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok());
            if let Some(command) = command {
                if let Some(id) = command["id"].as_u64().filter(|id| *id > seen) {
                    seen = id;
                    let label = command["window"].as_str().unwrap_or("main");
                    if let Some(window) = app.get_webview_window(label) {
                        let result = match command["native"].as_str() {
                            Some("publish-state") => app
                                .emit_to(label, "panel-state", command["payload"].clone())
                                .map_err(|error| error.to_string()),
                            Some("open-main") => commands::show_main(app.clone(), None, false),
                            Some("compact") => window
                                .show()
                                .and_then(|()| window.set_focus())
                                .map_err(|error| error.to_string()),
                            Some("resize") => window
                                .show()
                                .and_then(|()| window.set_focus())
                                .and_then(|()| {
                                    command["decorations"]
                                        .as_bool()
                                        .map_or(Ok(()), |enabled| window.set_decorations(enabled))
                                })
                                .and_then(|()| {
                                    window
                                        .set_position(tauri::LogicalPosition::new(10.0, 30.0))
                                        .and_then(|()| {
                                            window.set_size(tauri::LogicalSize::new(
                                                command["width"].as_f64().unwrap_or(1000.0),
                                                command["height"].as_f64().unwrap_or(700.0),
                                            ))
                                        })
                                })
                                .map_err(|error| error.to_string()),
                            Some("quit" | "close-main") => {
                                let _ = write_response(
                                    &driver,
                                    id,
                                    json!({"payload":{"ok":true,"value":null}}),
                                );
                                if command["native"] == "quit" {
                                    app.exit(0);
                                } else {
                                    let _ = window.close();
                                }
                                continue;
                            }
                            _ => Ok(()),
                        };
                        if let Err(error) = result {
                            let _ = write_response(
                                &driver,
                                id,
                                json!({"payload":{"ok":false,"error":error}}),
                            );
                            continue;
                        }
                        let script = command["script"].as_str().unwrap_or("return null;");
                        let script = format!(
                            r#"(async()=>{{try{{await document.fonts.ready;const value=await(async()=>{{{script}}})();await new Promise(r=>{{setTimeout(r,150);requestAnimationFrame(()=>requestAnimationFrame(r));}});await window.__TAURI__.core.invoke('test_report',{{id:{id},payload:{{ok:true,value}}}});}}catch(error){{await window.__TAURI__.core.invoke('test_report',{{id:{id},payload:{{ok:false,error:String(error)}}}});}}}})()"#
                        );
                        if let Err(error) = window.eval(&script) {
                            let _ = write_response(
                                &driver,
                                id,
                                json!({"payload":{"ok":false,"error":error.to_string()}}),
                            );
                        }
                    }
                }
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    });

    Ok(())
}

pub fn fixed_clock() -> Result<Option<u64>, std::num::ParseIntError> {
    std::env::var("ANYQ_PANEL_TEST_CLOCK")
        .ok()
        .map(|value| value.parse())
        .transpose()
}

pub fn clock_plugin() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    let script = if std::env::var_os("ANYQ_PANEL_TEST_CLOCK").is_some() {
        "(()=>{const original=Date.prototype.toLocaleTimeString;Date.prototype.toLocaleTimeString=function(locales,options){return original.call(this,locales,{...options,timeZone:'UTC'});};})()"
    } else {
        ""
    };
    tauri::plugin::Builder::new("desktop-test-clock")
        .js_init_script(script.to_owned())
        .build()
}
