#![forbid(unsafe_code)]

mod embedded_backend;

use std::{path::PathBuf, process::Command, sync::Arc};

use anyhow::Context;
use embedded_backend::DesktopRuntime;
use serde::Serialize;
use tauri::{Manager, State};

#[derive(Clone)]
struct RuntimeState {
    runtime: Arc<DesktopRuntime>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RuntimeInfo {
    base_url: String,
    desktop_token: String,
}

#[allow(non_snake_case)]
#[tauri::command]
fn runtimeInfo(state: State<'_, RuntimeState>) -> RuntimeInfo {
    RuntimeInfo {
        base_url: state.runtime.base_url().to_string(),
        desktop_token: state.runtime.desktop_token().to_string(),
    }
}

#[allow(non_snake_case)]
#[tauri::command]
fn openPath(path: String) -> Result<(), String> {
    open_native_path(PathBuf::from(path)).map_err(|err| err.to_string())
}

#[allow(non_snake_case)]
#[tauri::command]
fn pickDirectory() -> Option<String> {
    rfd::FileDialog::new()
        .pick_folder()
        .map(|path| path.to_string_lossy().into_owned())
}

#[allow(non_snake_case)]
#[tauri::command]
fn pickFile() -> Option<String> {
    rfd::FileDialog::new()
        .pick_file()
        .map(|path| path.to_string_lossy().into_owned())
}

pub fn run() -> anyhow::Result<()> {
    let runtime = Arc::new(DesktopRuntime::start().context("start embedded aio backend")?);
    tauri::Builder::default()
        .manage(RuntimeState { runtime })
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_title("AIO");
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            runtimeInfo,
            openPath,
            pickDirectory,
            pickFile
        ])
        .run(tauri::generate_context!())
        .map_err(|err| anyhow::anyhow!(err))
}

fn open_native_path(path: PathBuf) -> anyhow::Result<()> {
    #[cfg(target_os = "macos")]
    let mut command = Command::new("open");
    #[cfg(target_os = "linux")]
    let mut command = Command::new("xdg-open");
    #[cfg(target_os = "windows")]
    let mut command = {
        let mut command = Command::new("cmd");
        command.arg("/C").arg("start");
        command
    };

    command.arg(&path);
    command
        .spawn()
        .with_context(|| format!("open path {}", path.display()))?;
    Ok(())
}
