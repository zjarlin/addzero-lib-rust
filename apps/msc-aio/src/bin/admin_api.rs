#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    msc_aio::server::run_api_server().await
}

#[cfg(target_arch = "wasm32")]
fn main() {}
