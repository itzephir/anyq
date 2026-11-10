//! Private, feature-gated native UI test transport; absent from shipping builds.
use crate::{commands, controller::Controller};
use serde_json::{Value, json};
use std::{fs, path::PathBuf, sync::Arc, time::Duration};
use tauri::Manager;

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

    write_response(
        &driver,
        id,
        json!({ "payload": payload, "rect": {"x":position.x,"y":position.y,"width":size.width,"height":size.height,"scale":scale}, "tray":tray }),
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
            let command = fs::read(driver.directory.join("command.json"))
                .ok()
                .filter(|bytes| bytes.len() <= 65_536)
                .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok());
            if let Some(command) = command {
                if let Some(id) = command["id"].as_u64().filter(|id| *id > seen) {
                    seen = id;
                    let label = command["window"].as_str().unwrap_or("main");
                    if let Some(window) = app.get_webview_window(label) {
                        let result = match command["native"].as_str() {
                            Some("open-main") => commands::show_main(app.clone(), None, false),
                            Some("compact") => window
                                .show()
                                .and_then(|()| window.set_focus())
                                .map_err(|error| error.to_string()),
                            Some("resize") => window
                                .set_position(tauri::LogicalPosition::new(10.0, 30.0))
                                .and_then(|()| {
                                    window.set_size(tauri::LogicalSize::new(
                                        command["width"].as_f64().unwrap_or(1000.0),
                                        command["height"].as_f64().unwrap_or(700.0),
                                    ))
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
