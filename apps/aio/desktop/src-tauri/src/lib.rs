use std::time::Duration;

/// Start the axum API server in a background tokio task, then block until it
/// responds to health checks (or time out after 10 seconds).
fn start_api_server() {
    let api_base =
        std::env::var("AIO_API_BASE").unwrap_or_else(|_| "http://127.0.0.1:8787".to_string());
    let health_url = format!("{api_base}/api/admin/session");

    // Spawn the server. It binds to AIO_API_BIND (default 127.0.0.1:8787).
    tauri::async_runtime::spawn(async {
        log::info!("Starting API server...");
        if let Err(e) = aio::server::run_api_server().await {
            log::error!("API server exited with error: {e}");
        }
    });

    // Busy-wait with a short sleep until the health endpoint responds.
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .expect("health-check client");

    for i in 0..50 {
        match client.get(&health_url).send() {
            Ok(_) => {
                log::info!("API server ready at {api_base}");
                return;
            }
            Err(e) if i % 10 == 0 => {
                log::info!("Waiting for API server... ({e})");
            }
            _ => {}
        }
        std::thread::sleep(Duration::from_millis(200));
    }

    log::error!("API server did not become ready within 10 seconds");
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|_app| {
            start_api_server();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
