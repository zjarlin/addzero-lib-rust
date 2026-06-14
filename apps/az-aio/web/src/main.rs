#![forbid(unsafe_code)]

mod app;

use anyhow::Result;
use axum::{Router, routing::get};
use std::env;
use std::net::SocketAddr;
use tower_http::services::ServeDir;

const DEFAULT_PORT: u16 = 8080;
const PORT_ENV: &str = "AZ_AIO_WEB_PORT";
const CONFIG_CENTER_URL_ENV: &str = "AZ_CONFIG_CENTER_BASE_URL";
const CONFIG_NAMESPACE: &str = "az-aio.dev";
const CONFIG_KEY_PORT: &str = "web.port";

#[tokio::main]
async fn main() -> Result<()> {
    az_aio_plugin_bundled::api::ensure_linked();

    let snapshot = az_aio_plugin_host::host::load_az_aio_native_snapshot();
    let _loopback_url = az_aio_plugin_host::host::start_native_loopback_server(snapshot.clone())
        .await
        .ok();

    let state = AppState { snapshot };

    let app = Router::new()
        .route("/", get(root_page))
        .route("/health", get(health))
        .nest_service("/assets", ServeDir::new("assets"))
        .with_state(state);

    let port = resolve_port();
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("AZ AIO web workbench listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn resolve_port() -> u16 {
    if let Some(port) = port_from_config_center() {
        return port;
    }
    if let Some(port) = port_from_env() {
        return port;
    }
    println!("no config-center or {PORT_ENV} set, defaulting to port {DEFAULT_PORT}");
    DEFAULT_PORT
}

fn port_from_config_center() -> Option<u16> {
    let base_url = env::var(CONFIG_CENTER_URL_ENV).ok()?;
    let username = env::var("AZ_CONFIG_CENTER_USERNAME").unwrap_or_default();
    let password = env::var("AZ_CONFIG_CENTER_PASSWORD").unwrap_or_default();

    if username.is_empty() || password.is_empty() {
        return None;
    }

    let client = az_config_center_client::ConfigCenterClient::new(&base_url)
        .ok()?
        .login(&username, &password)
        .ok()?
        .checkout_namespace(CONFIG_NAMESPACE)
        .ok()?;

    let port_str: String = client.get_text(CONFIG_KEY_PORT).ok()??;
    port_str.trim().parse().ok()
}

fn port_from_env() -> Option<u16> {
    env::var(PORT_ENV).ok().and_then(|v| v.trim().parse().ok())
}

#[derive(Clone)]
struct AppState {
    snapshot: az_aio_plugin_host::host::HostSnapshot,
}

async fn root_page(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> axum::response::Html<String> {
    let html = app::render_app_html(&state.snapshot);
    axum::response::Html(html)
}

async fn health() -> &'static str {
    "ok"
}
