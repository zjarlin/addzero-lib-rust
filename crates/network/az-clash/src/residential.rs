//! Residential proxy via local Clash/Mihomo process.
//!
//! Fetches proxy subscriptions, speed-tests nodes, selects the fastest one,
//! and starts a local Clash process to provide an HTTP/SOCKS5 proxy.
//!
//! ```no_run
//! # async fn run() -> az_clash::ClashResult<()> {
//! let config = az_clash::ResidentialProxyConfig::new("https://example.com/sub");
//! let proxy = az_clash::ResidentialProxy::start(config).await?;
//! println!("Residential proxy ready at {}", proxy.http_proxy);
//! // proxy.http_proxy is "http://127.0.0.1:7890"
//! // proxy.socks5_proxy() is "socks5://127.0.0.1:7890"
//! # Ok(())
//! # }
//! ```

use crate::fetcher::fetch_and_parse;
use crate::selector::{generate_clash_config, select_fastest_node};
use crate::speedtest::batch_speed_test;
use crate::types::{ClashError, ClashResult, ProxyNode};
use az_derive_aliases::{apply, plain_clone_debug};
use std::fs;
use std::net::TcpStream;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

/// Configuration for starting a residential proxy via local Clash.
#[apply(plain_clone_debug)]
pub struct ResidentialProxyConfig {
    /// Subscription URL to fetch proxy nodes from.
    pub subscription_url: String,
    /// Local HTTP/SOCKS mixed port (default: [`crate::DEFAULT_MIXED_PORT`] = 7890).
    pub mixed_port: u16,
    /// Speed test concurrency (default: [`crate::DEFAULT_SPEEDTEST_CONCURRENCY`] = 10).
    pub speedtest_concurrency: usize,
    /// Speed test per-node timeout (default: [`crate::DEFAULT_SPEEDTEST_TIMEOUT`] = 5s).
    pub speedtest_timeout: Duration,
    /// Explicit Clash/Mihomo binary path. Auto-detected when `None`.
    pub clash_binary: Option<PathBuf>,
    /// Maximum time to wait for the proxy port to become ready.
    pub ready_timeout: Duration,
}

impl ResidentialProxyConfig {
    /// Creates a configuration with all defaults for the given subscription URL.
    #[must_use]
    pub fn new(subscription_url: impl Into<String>) -> Self {
        Self {
            subscription_url: subscription_url.into(),
            mixed_port: crate::DEFAULT_MIXED_PORT,
            speedtest_concurrency: crate::DEFAULT_SPEEDTEST_CONCURRENCY,
            speedtest_timeout: crate::DEFAULT_SPEEDTEST_TIMEOUT,
            clash_binary: None,
            ready_timeout: Duration::from_secs(15),
        }
    }

    /// Uses an explicit Clash/Mihomo binary path.
    #[must_use]
    pub fn with_clash_binary(mut self, path: impl Into<PathBuf>) -> Self {
        self.clash_binary = Some(path.into());
        self
    }

    /// Overrides the local mixed proxy port.
    #[must_use]
    pub fn with_port(mut self, port: u16) -> Self {
        self.mixed_port = port;
        self
    }

    /// Overrides the speed test concurrency.
    #[must_use]
    pub fn with_concurrency(mut self, concurrency: usize) -> Self {
        self.speedtest_concurrency = concurrency;
        self
    }

    /// Overrides the speed test timeout.
    #[must_use]
    pub fn with_speedtest_timeout(mut self, timeout: Duration) -> Self {
        self.speedtest_timeout = timeout;
        self
    }
}

/// A running residential proxy backed by a local Clash process.
///
/// The Clash process is killed and the temporary config file removed on drop.
pub struct ResidentialProxy {
    /// Selected proxy node being used.
    pub node: ProxyNode,
    /// Local HTTP proxy URL (e.g. `http://127.0.0.1:7890`).
    pub http_proxy: String,
    /// Local proxy port.
    pub port: u16,
    child: Option<Child>,
    config_path: Option<PathBuf>,
}

impl ResidentialProxy {
    /// Fetches the subscription, tests nodes, selects the fastest, starts Clash,
    /// and returns a ready local proxy.
    ///
    /// # Errors
    ///
    /// Returns an error when fetching/parsing fails, no node passes the speed
    /// test, the Clash binary cannot be found, or the proxy port does not become
    /// ready within `config.ready_timeout`.
    pub async fn start(config: ResidentialProxyConfig) -> ClashResult<Self> {
        let nodes = fetch_and_parse(&config.subscription_url).await?;

        let results = batch_speed_test(
            &nodes,
            config.speedtest_concurrency,
            config.speedtest_timeout,
        )
        .await;

        let node = select_fastest_node(&nodes, &results)?.clone();

        let clash_yaml = generate_clash_config(&node, config.mixed_port)?;
        let config_path = write_temp_config(&clash_yaml)?;

        let clash_binary = config
            .clash_binary
            .clone()
            .or_else(find_clash_binary)
            .ok_or(ClashError::ClashBinaryNotFound)?;

        let work_dir = config_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("/tmp"));

        let mut child = Command::new(&clash_binary)
            .arg("-f")
            .arg(&config_path)
            .arg("-d")
            .arg(work_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| ClashError::ClashProcess(format!("failed to start clash: {error}")))?;

        if !wait_for_port(config.mixed_port, config.ready_timeout) {
            let _ = child.kill();
            let _ = child.wait();
            let _ = fs::remove_file(&config_path);
            return Err(ClashError::ClashProcess(format!(
                "proxy port {} not ready after {}s",
                config.mixed_port,
                config.ready_timeout.as_secs()
            )));
        }

        Ok(Self {
            node,
            http_proxy: format!("http://127.0.0.1:{}", config.mixed_port),
            port: config.mixed_port,
            child: Some(child),
            config_path: Some(config_path),
        })
    }

    /// Returns the SOCKS5 proxy URL on the same port.
    #[must_use]
    pub fn socks5_proxy(&self) -> String {
        format!("socks5://127.0.0.1:{}", self.port)
    }
}

impl Drop for ResidentialProxy {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        if let Some(ref path) = self.config_path {
            let _ = fs::remove_file(path);
        }
    }
}

fn write_temp_config(yaml: &str) -> ClashResult<PathBuf> {
    let dir = std::env::temp_dir().join("az-clash");
    fs::create_dir_all(&dir)
        .map_err(|error| ClashError::ClashProcess(format!("create temp dir: {error}")))?;
    let path = dir.join(format!("residential-{}.yaml", std::process::id()));
    fs::write(&path, yaml)
        .map_err(|error| ClashError::ClashProcess(format!("write config: {error}")))?;
    Ok(path)
}

fn find_clash_binary() -> Option<PathBuf> {
    for env_key in &["CLASH_BINARY", "MIHOMO_HOME"] {
        if let Ok(value) = std::env::var(env_key) {
            let path = PathBuf::from(&value);
            if path.exists() {
                return Some(path);
            }
            if let Ok(joined) = PathBuf::from(&value).join("mihomo").canonicalize() {
                return Some(joined);
            }
        }
    }

    let candidates: &[&str] = &[
        "/usr/local/bin/mihomo",
        "/usr/local/bin/clash",
        "/opt/homebrew/bin/mihomo",
        "/opt/homebrew/bin/clash",
        "/usr/bin/mihomo",
        "/usr/bin/clash",
    ];

    for candidate in candidates {
        let path = PathBuf::from(candidate);
        if path.exists() {
            return Some(path);
        }
    }

    if let Some(home) = dirs::home_dir() {
        for rel in &[".local/bin/mihomo", ".local/bin/clash"] {
            let path = home.join(rel);
            if path.exists() {
                return Some(path);
            }
        }
    }

    None
}

fn wait_for_port(port: u16, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if TcpStream::connect(format!("127.0.0.1:{port}")).is_ok() {
            return true;
        }
        thread::sleep(Duration::from_millis(300));
    }
    false
}
