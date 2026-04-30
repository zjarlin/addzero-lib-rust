use addzero_remote_session::RelayRuntimeConfig;
use tokio::time::{Duration, sleep};

#[tokio::main]
async fn main() {
    let config = RelayRuntimeConfig::default();
    println!(
        "remote relay listening on {} with max {} concurrent sessions",
        config.bind_addr, config.max_concurrent_sessions
    );
    loop {
        println!(
            "relay heartbeat: idle_timeout={}s",
            config.idle_timeout_secs
        );
        sleep(Duration::from_secs(30)).await;
    }
}
