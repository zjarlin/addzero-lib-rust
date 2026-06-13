//! 通过本地 Clash/Mihomo 进程提供住宅代理。
//!
//! 本模块会获取代理订阅、测速节点、选择最快节点，并启动本地 Clash 进程提供
//! HTTP/SOCKS5 代理。
//!
//! ```no_run
//! # async fn run() -> anyhow::Result<()> {
//! let config = az_proxy::clash::ResidentialProxyConfig::new("https://example.com/sub");
//! let proxy = az_proxy::clash::ResidentialProxy::start(config).await?;
//! println!("Residential proxy ready at {}", proxy.http_proxy);
//! // proxy.http_proxy 为 "http://127.0.0.1:7890"
//! // proxy.socks5_proxy() 为 "socks5://127.0.0.1:7890"
//! # Ok(())
//! # }
//! ```

use super::{DEFAULT_MIXED_PORT, generate_clash_config};
use crate::fetcher::fetch_and_parse;
use crate::selector::select_fastest_node;
use crate::speedtest::batch_speed_test;
use crate::types::{
    DEFAULT_SPEEDTEST_CONCURRENCY, DEFAULT_SPEEDTEST_TIMEOUT, ProxyNode,
};
use anyhow::{Context, Result, bail};
use az_derive_aliases::{apply, plain_clone_debug};
use std::fs;
use std::net::TcpStream;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

/// 通过本地 Clash 启动住宅代理的配置。
#[apply(plain_clone_debug)]
pub struct ResidentialProxyConfig {
    /// 用于获取代理节点的订阅 URL。
    pub subscription_url: String,
    /// 本地 HTTP/SOCKS 混合端口，默认 [`DEFAULT_MIXED_PORT`] = 7890。
    pub mixed_port: u16,
    /// 测速并发数，默认 [`crate::types::DEFAULT_SPEEDTEST_CONCURRENCY`] = 10。
    pub speedtest_concurrency: usize,
    /// 单节点测速超时时间，默认 [`crate::types::DEFAULT_SPEEDTEST_TIMEOUT`] = 5s。
    pub speedtest_timeout: Duration,
    /// 显式 Clash/Mihomo 可执行文件路径；为 `None` 时自动探测。
    pub clash_binary: Option<PathBuf>,
    /// 等待代理端口就绪的最长时间。
    pub ready_timeout: Duration,
}

impl ResidentialProxyConfig {
    /// 使用指定订阅 URL 和默认参数创建配置。
    #[must_use]
    pub fn new(subscription_url: impl Into<String>) -> Self {
        Self {
            subscription_url: subscription_url.into(),
            mixed_port: DEFAULT_MIXED_PORT,
            speedtest_concurrency: DEFAULT_SPEEDTEST_CONCURRENCY,
            speedtest_timeout: DEFAULT_SPEEDTEST_TIMEOUT,
            clash_binary: None,
            ready_timeout: Duration::from_secs(15),
        }
    }

    /// 使用显式 Clash/Mihomo 可执行文件路径。
    #[must_use]
    pub fn with_clash_binary(mut self, path: impl Into<PathBuf>) -> Self {
        self.clash_binary = Some(path.into());
        self
    }

    /// 覆盖本地 mixed 代理端口。
    #[must_use]
    pub fn with_port(mut self, port: u16) -> Self {
        self.mixed_port = port;
        self
    }

    /// 覆盖测速并发数。
    #[must_use]
    pub fn with_concurrency(mut self, concurrency: usize) -> Self {
        self.speedtest_concurrency = concurrency;
        self
    }

    /// 覆盖测速超时时间。
    #[must_use]
    pub fn with_speedtest_timeout(mut self, timeout: Duration) -> Self {
        self.speedtest_timeout = timeout;
        self
    }
}

/// 正在运行的本地 Clash 住宅代理。
///
/// 该值 drop 时会结束 Clash 进程并删除临时配置文件。
pub struct ResidentialProxy {
    /// 当前选中的代理节点。
    pub node: ProxyNode,
    /// 本地 HTTP 代理 URL，例如 `http://127.0.0.1:7890`。
    pub http_proxy: String,
    /// 本地代理端口。
    pub port: u16,
    child: Option<Child>,
    config_path: Option<PathBuf>,
}

impl ResidentialProxy {
    /// 获取订阅、测试节点、选择最快节点、启动 Clash，并返回就绪的本地代理。
    ///
    /// # Errors
    ///
    /// 当获取/解析失败、没有节点通过测速、找不到 Clash 可执行文件，或代理端口没有在
    /// `config.ready_timeout` 内就绪时返回错误。
    pub async fn start(config: ResidentialProxyConfig) -> Result<Self> {
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
            .context("could not find Clash/Mihomo binary; set CLASH_BINARY env var")?;

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
            .with_context(|| format!("failed to start clash `{}`", clash_binary.display()))?;

        if !wait_for_port(config.mixed_port, config.ready_timeout) {
            let _ = child.kill();
            let _ = child.wait();
            let _ = fs::remove_file(&config_path);
            bail!(
                "proxy port {} not ready after {}s",
                config.mixed_port,
                config.ready_timeout.as_secs()
            );
        }

        Ok(Self {
            node,
            http_proxy: format!("http://127.0.0.1:{}", config.mixed_port),
            port: config.mixed_port,
            child: Some(child),
            config_path: Some(config_path),
        })
    }

    /// 返回同一端口上的 SOCKS5 代理 URL。
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

fn write_temp_config(yaml: &str) -> Result<PathBuf> {
    let dir = std::env::temp_dir().join("az-proxy");
    fs::create_dir_all(&dir)
        .with_context(|| format!("create Clash temp config dir `{}`", dir.display()))?;
    let path = dir.join(format!("residential-{}.yaml", std::process::id()));
    fs::write(&path, yaml)
        .with_context(|| format!("write Clash temp config `{}`", path.display()))?;
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
