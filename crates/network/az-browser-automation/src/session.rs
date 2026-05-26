//! Isolated browser sessions backed by a dedicated Chrome process.

use crate::browser_automation::{
    BrowserAutomationError, BrowserAutomationResult, query_cdp_websocket_url,
    resolve_chrome_executable,
};
use crate::fingerprint::FingerprintProfile;
use crate::proxy::ProxyConfig;
use az_derive_aliases::{apply, plain_default_eq, plain_eq};
use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption;
use headless_chrome::protocol::cdp::{Accessibility, Runtime};
use headless_chrome::{Browser, Tab};
use rand::Rng;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicU16, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use uuid::Uuid;

static NEXT_CDP_PORT: AtomicU16 = AtomicU16::new(9300);

/// Strategy used to choose the browser profile for a session.
#[apply(plain_default_eq)]
pub enum FingerprintStrategy {
    /// Pick a random profile from the built-in profile pool.
    #[default]
    Random,
    /// Use one explicit profile.
    Specific(FingerprintProfile),
    /// Pick a random profile from a caller-supplied pool.
    Pool(Vec<FingerprintProfile>),
}

impl FingerprintStrategy {
    fn resolve(&self) -> BrowserAutomationResult<FingerprintProfile> {
        match self {
            Self::Random => Ok(FingerprintProfile::random()),
            Self::Specific(profile) => Ok(profile.clone()),
            Self::Pool(pool) => {
                if pool.is_empty() {
                    return Err(BrowserAutomationError::InvalidSessionConfig(
                        "fingerprint pool must contain at least one profile".to_owned(),
                    ));
                }
                let mut rng = rand::thread_rng();
                let index = rng.gen_range(0..pool.len());
                Ok(pool[index].clone())
            }
        }
    }
}

/// Configuration for an isolated [`BrowserSession`].
#[apply(plain_eq)]
pub struct SessionConfig {
    /// Browser profile selection strategy.
    pub fingerprint: FingerprintStrategy,
    /// Optional proxy used when launching Chrome.
    pub proxy: Option<ProxyConfig>,
    /// Optional fixed CDP port. When omitted, ports start at `9300` and increment.
    pub cdp_port: Option<u16>,
    /// Optional Chrome user-data directory. Caller-owned directories are not removed on drop.
    pub user_data_dir: Option<PathBuf>,
    /// Whether Chrome should run headless.
    pub headless: bool,
    /// Session startup and browser operation timeout in milliseconds.
    pub timeout_ms: u64,
    /// Optional explicit Chrome or Chromium executable.
    pub executable_path: Option<PathBuf>,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            fingerprint: FingerprintStrategy::Random,
            proxy: None,
            cdp_port: None,
            user_data_dir: None,
            headless: true,
            timeout_ms: 30_000,
            executable_path: None,
        }
    }
}

impl SessionConfig {
    /// Creates a builder initialized with default session options.
    #[must_use]
    pub fn builder() -> SessionConfigBuilder {
        SessionConfigBuilder::default()
    }
}

/// Fluent builder for [`SessionConfig`].
#[apply(plain_default_eq)]
pub struct SessionConfigBuilder {
    config: SessionConfig,
}

impl SessionConfigBuilder {
    /// Sets the fingerprint strategy.
    #[must_use]
    pub fn fingerprint(mut self, fingerprint: FingerprintStrategy) -> Self {
        self.config.fingerprint = fingerprint;
        self
    }

    /// Uses an explicit fingerprint profile.
    #[must_use]
    pub fn fingerprint_profile(mut self, profile: FingerprintProfile) -> Self {
        self.config.fingerprint = FingerprintStrategy::Specific(profile);
        self
    }

    /// Uses a caller-supplied fingerprint profile pool.
    #[must_use]
    pub fn fingerprint_pool(mut self, pool: Vec<FingerprintProfile>) -> Self {
        self.config.fingerprint = FingerprintStrategy::Pool(pool);
        self
    }

    /// Sets the proxy configuration.
    #[must_use]
    pub fn proxy(mut self, proxy: ProxyConfig) -> Self {
        self.config.proxy = Some(proxy);
        self
    }

    /// Sets a fixed CDP port.
    #[must_use]
    pub fn cdp_port(mut self, port: u16) -> Self {
        self.config.cdp_port = Some(port);
        self
    }

    /// Sets a fixed user-data directory.
    #[must_use]
    pub fn user_data_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.config.user_data_dir = Some(path.into());
        self
    }

    /// Sets whether Chrome should run headless.
    #[must_use]
    pub fn headless(mut self, value: bool) -> Self {
        self.config.headless = value;
        self
    }

    /// Sets the session timeout in milliseconds.
    #[must_use]
    pub fn timeout_ms(mut self, value: u64) -> Self {
        self.config.timeout_ms = value;
        self
    }

    /// Sets an explicit Chrome or Chromium executable path.
    #[must_use]
    pub fn executable_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.config.executable_path = Some(path.into());
        self
    }

    /// Finishes the builder and returns the session configuration.
    #[must_use]
    pub fn build(self) -> SessionConfig {
        self.config
    }
}

/// Dedicated browser automation session with isolated Chrome process state.
pub struct BrowserSession {
    /// Browser profile applied before navigation.
    pub fingerprint: FingerprintProfile,
    /// Optional proxy used to launch the Chrome process.
    pub proxy: Option<ProxyConfig>,
    /// Chrome DevTools Protocol port for this session.
    pub cdp_port: u16,
    /// Chrome user-data directory for this session.
    pub user_data_dir: PathBuf,
    browser: Browser,
    tab: Arc<Tab>,
    child: Option<Child>,
    cleanup_user_data_dir: bool,
    timeout_ms: u64,
}

impl BrowserSession {
    /// Starts a new isolated Chrome process and connects through CDP.
    ///
    /// # Errors
    ///
    /// Returns [`BrowserAutomationError`] if the profile strategy is invalid,
    /// Chrome cannot be found or started, CDP does not become ready, or the
    /// initial tab cannot be created.
    pub fn new(config: SessionConfig) -> BrowserAutomationResult<Self> {
        let fingerprint = config.fingerprint.resolve()?;
        let cdp_port = config.cdp_port.unwrap_or_else(next_cdp_port);
        let chrome_path = config
            .executable_path
            .or_else(resolve_chrome_executable)
            .ok_or(BrowserAutomationError::ChromeExecutableNotFound)?;
        let (user_data_dir, cleanup_user_data_dir) = session_profile_dir(config.user_data_dir)?;
        let mut command = Command::new(chrome_path);

        command
            .arg(format!("--remote-debugging-port={cdp_port}"))
            .arg(format!("--user-data-dir={}", user_data_dir.display()))
            .arg("--no-first-run")
            .arg("--no-default-browser-check")
            .arg("--disable-background-networking")
            .arg("--password-store=basic")
            .arg("--no-sandbox")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        if config.headless {
            command.arg("--headless=new").arg("--disable-gpu");
        }

        if let Some(proxy) = &config.proxy {
            command.arg(proxy.chrome_arg());
        }

        let mut child = command
            .spawn()
            .map_err(|error| BrowserAutomationError::ChromeLaunch(error.to_string()))?;
        let endpoint = format!("http://127.0.0.1:{cdp_port}");

        if let Err(error) = wait_for_cdp(&endpoint, config.timeout_ms) {
            cleanup_failed_session(&mut child, cleanup_user_data_dir, &user_data_dir);
            return Err(error);
        }

        let ws_url = match query_cdp_websocket_url(&endpoint, config.timeout_ms) {
            Ok(ws_url) => ws_url,
            Err(error) => {
                cleanup_failed_session(&mut child, cleanup_user_data_dir, &user_data_dir);
                return Err(error);
            }
        };
        let browser =
            match Browser::connect_with_timeout(ws_url, Duration::from_millis(config.timeout_ms)) {
                Ok(browser) => browser,
                Err(error) => {
                    cleanup_failed_session(&mut child, cleanup_user_data_dir, &user_data_dir);
                    return Err(BrowserAutomationError::Browser(error.to_string()));
                }
            };
        let tab = match browser.new_tab() {
            Ok(tab) => tab,
            Err(error) => {
                cleanup_failed_session(&mut child, cleanup_user_data_dir, &user_data_dir);
                return Err(BrowserAutomationError::Browser(error.to_string()));
            }
        };
        tab.set_default_timeout(Duration::from_millis(config.timeout_ms));
        if let Err(error) = fingerprint.inject(tab.as_ref()) {
            cleanup_failed_session(&mut child, cleanup_user_data_dir, &user_data_dir);
            return Err(error);
        }

        Ok(Self {
            fingerprint,
            proxy: config.proxy,
            cdp_port,
            user_data_dir,
            browser,
            tab,
            child: Some(child),
            cleanup_user_data_dir,
            timeout_ms: config.timeout_ms,
        })
    }

    /// Starts a new session with a proxy and default options.
    ///
    /// # Errors
    ///
    /// Returns the same errors as [`BrowserSession::new`].
    pub fn with_proxy(proxy: ProxyConfig) -> BrowserAutomationResult<Self> {
        Self::new(SessionConfig::builder().proxy(proxy).build())
    }

    /// Returns the underlying `headless_chrome` browser handle.
    #[must_use]
    pub fn browser(&self) -> &Browser {
        &self.browser
    }

    /// Returns the session's active tab.
    #[must_use]
    pub fn tab(&self) -> &Arc<Tab> {
        &self.tab
    }

    /// Navigates the active tab to `url`.
    ///
    /// # Errors
    ///
    /// Returns [`BrowserAutomationError::Browser`] if navigation or the
    /// navigation wait fails.
    pub fn navigate(&self, url: &str) -> BrowserAutomationResult<()> {
        self.tab
            .navigate_to(url)
            .and_then(|tab| tab.wait_until_navigated())
            .map_err(|error| BrowserAutomationError::Browser(error.to_string()))?;
        Ok(())
    }

    /// Returns the active page accessibility tree as formatted JSON.
    ///
    /// # Errors
    ///
    /// Returns [`BrowserAutomationError::Browser`] if CDP access or JSON
    /// serialization fails.
    pub fn snapshot(&self) -> BrowserAutomationResult<String> {
        let tree = self
            .tab
            .call_method(Accessibility::GetFullAXTree {
                depth: None,
                frame_id: None,
            })
            .map_err(|error| BrowserAutomationError::Browser(error.to_string()))?;
        serde_json::to_string_pretty(&tree.nodes)
            .map_err(|error| BrowserAutomationError::Browser(error.to_string()))
    }

    /// Executes JavaScript in the active tab.
    ///
    /// # Errors
    ///
    /// Returns [`BrowserAutomationError::Browser`] if evaluation fails.
    pub fn execute_js(&self, js: &str) -> BrowserAutomationResult<Value> {
        let result = self
            .tab
            .call_method(Runtime::Evaluate {
                expression: js.to_owned(),
                object_group: None,
                include_command_line_api: Some(false),
                silent: Some(false),
                context_id: None,
                return_by_value: Some(true),
                generate_preview: Some(false),
                user_gesture: Some(false),
                await_promise: Some(true),
                throw_on_side_effect: None,
                timeout: None,
                disable_breaks: None,
                repl_mode: None,
                allow_unsafe_eval_blocked_by_csp: None,
                unique_context_id: None,
                serialization_options: None,
            })
            .map_err(|error| BrowserAutomationError::Browser(error.to_string()))?;
        if let Some(exception) = result.exception_details {
            return Err(BrowserAutomationError::Browser(format!(
                "JavaScript evaluation failed: {exception:?}"
            )));
        }
        let value = result.result.value.unwrap_or(Value::Null);
        Ok(value)
    }

    /// Captures a PNG screenshot at `path`.
    ///
    /// # Errors
    ///
    /// Returns [`BrowserAutomationError`] when screenshot capture or file write
    /// fails.
    pub fn screenshot(&self, path: impl AsRef<Path>) -> BrowserAutomationResult<PathBuf> {
        let path = path.as_ref();
        let screenshot = self
            .tab
            .capture_screenshot(CaptureScreenshotFormatOption::Png, None, None, true)
            .map_err(|error| BrowserAutomationError::Browser(error.to_string()))?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| BrowserAutomationError::ArtifactIo {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        fs::write(path, screenshot).map_err(|source| BrowserAutomationError::ArtifactIo {
            path: path.to_path_buf(),
            source,
        })?;
        Ok(path.to_path_buf())
    }

    /// Returns this session's CDP HTTP endpoint.
    #[must_use]
    pub fn cdp_endpoint(&self) -> String {
        format!("http://127.0.0.1:{}", self.cdp_port)
    }

    /// Returns the session operation timeout.
    #[must_use]
    pub fn timeout(&self) -> Duration {
        Duration::from_millis(self.timeout_ms)
    }
}

impl Drop for BrowserSession {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            if let Err(error) = child.kill() {
                tracing::debug!(%error, port = self.cdp_port, "Chrome process already stopped");
            }
            if let Err(error) = child.wait() {
                tracing::debug!(%error, port = self.cdp_port, "failed to wait for Chrome process");
            }
        }

        if self.cleanup_user_data_dir
            && let Err(error) = fs::remove_dir_all(&self.user_data_dir)
        {
            tracing::debug!(
                %error,
                path = %self.user_data_dir.display(),
                "failed to remove browser session directory"
            );
        }
    }
}

fn next_cdp_port() -> u16 {
    NEXT_CDP_PORT.fetch_add(1, Ordering::Relaxed)
}

fn session_profile_dir(path: Option<PathBuf>) -> BrowserAutomationResult<(PathBuf, bool)> {
    match path {
        Some(path) => {
            fs::create_dir_all(&path)
                .map_err(|error| BrowserAutomationError::ChromeLaunch(error.to_string()))?;
            Ok((path, false))
        }
        None => {
            let path = std::env::temp_dir().join(format!("az-browser-session-{}", Uuid::new_v4()));
            fs::create_dir_all(&path)
                .map_err(|error| BrowserAutomationError::ChromeLaunch(error.to_string()))?;
            Ok((path, true))
        }
    }
}

fn wait_for_cdp(endpoint: &str, timeout_ms: u64) -> BrowserAutomationResult<()> {
    let deadline = Instant::now() + Duration::from_millis(timeout_ms);
    while Instant::now() < deadline {
        if query_cdp_websocket_url(endpoint, 1_500).is_ok() {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(250));
    }
    Err(BrowserAutomationError::SessionNotReady {
        endpoint: endpoint.to_owned(),
        timeout_ms,
    })
}

fn cleanup_failed_session(child: &mut Child, cleanup_user_data_dir: bool, user_data_dir: &Path) {
    if let Err(error) = child.kill() {
        tracing::debug!(%error, "Chrome process already stopped during failed session cleanup");
    }
    if let Err(error) = child.wait() {
        tracing::debug!(%error, "failed to wait during failed session cleanup");
    }
    if cleanup_user_data_dir && let Err(error) = fs::remove_dir_all(user_data_dir) {
        tracing::debug!(
            %error,
            path = %user_data_dir.display(),
            "failed to remove failed browser session directory"
        );
    }
}
