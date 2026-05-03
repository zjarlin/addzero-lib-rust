#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Manager, Url};

const DEV_URL: &str = "http://127.0.0.1:3000";
const PROD_URL: &str = "http://127.0.0.1:43189";
const LOCALHOST_PORT: u16 = 43189;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_localhost::Builder::new(LOCALHOST_PORT).build())
        .setup(|app| {
            let window = app.get_webview_window("main").expect("main window");
            let url = if cfg!(debug_assertions) { DEV_URL } else { PROD_URL };
            window.navigate(Url::parse(url).expect("valid window url"))?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
