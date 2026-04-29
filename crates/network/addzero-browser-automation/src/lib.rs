use addzero_context::ThreadLocalUtil;
use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption;
use headless_chrome::{Browser, LaunchOptionsBuilder, Tab};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;

pub type BrowserAutomationResult<T> = Result<T, BrowserAutomationError>;

#[derive(Debug, Error)]
pub enum BrowserAutomationError {
    #[error("browser launch configuration is invalid: {0}")]
    InvalidLaunchOptions(String),
    #[error("browser operation failed: {0}")]
    Browser(String),
    #[error("required field `{name}` was not found on `{url}`")]
    MissingRequiredField { name: String, url: String },
    #[error("browser context does not contain a start url")]
    MissingStartUrl,
    #[error("cdp endpoint `{0}` does not expose a websocket debugger url")]
    MissingCdpWebSocketUrl(String),
    #[error("failed to query cdp endpoint `{endpoint}`: {message}")]
    CdpEndpointQuery { endpoint: String, message: String },
    #[error("could not find a Chrome/Chromium executable for CDP mode")]
    ChromeExecutableNotFound,
    #[error("failed to start Chrome for CDP mode: {0}")]
    ChromeLaunch(String),
    #[error("failed to persist debug artifact at `{path}`: {source}")]
    ArtifactIo {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserAutomationOptions {
    pub debug: bool,
    pub headless: bool,
    pub timeout_ms: u64,
    pub slow_mo_ms: u64,
    pub artifacts_dir: Option<PathBuf>,
    pub executable_path: Option<PathBuf>,
    pub mode: BrowserMode,
}

impl Default for BrowserAutomationOptions {
    fn default() -> Self {
        Self {
            debug: false,
            headless: true,
            timeout_ms: 30_000,
            slow_mo_ms: 0,
            artifacts_dir: None,
            executable_path: None,
            mode: BrowserMode::default(),
        }
    }
}

impl BrowserAutomationOptions {
    pub fn effective_headless(&self) -> bool {
        if self.debug { false } else { self.headless }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BrowserMode {
    Cdp(CdpEndpoint),
    Launch,
}

impl Default for BrowserMode {
    fn default() -> Self {
        Self::Cdp(CdpEndpoint::default())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CdpEndpoint {
    Http(String),
    WebSocket(String),
}

impl Default for CdpEndpoint {
    fn default() -> Self {
        Self::Http("http://127.0.0.1:9222".to_owned())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FieldType {
    #[default]
    Input,
    Click,
    Check,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormFieldDef {
    pub name: String,
    pub selectors: Vec<String>,
    pub value: String,
    pub required: bool,
    pub field_type: FieldType,
}

impl FormFieldDef {
    pub fn input(
        name: impl Into<String>,
        selectors: impl IntoIterator<Item = impl Into<String>>,
        value: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            selectors: selectors.into_iter().map(Into::into).collect(),
            value: value.into(),
            required: false,
            field_type: FieldType::Input,
        }
    }

    pub fn click(
        name: impl Into<String>,
        selectors: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            name: name.into(),
            selectors: selectors.into_iter().map(Into::into).collect(),
            value: String::new(),
            required: false,
            field_type: FieldType::Click,
        }
    }

    pub fn check(
        name: impl Into<String>,
        selectors: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            name: name.into(),
            selectors: selectors.into_iter().map(Into::into).collect(),
            value: String::new(),
            required: false,
            field_type: FieldType::Check,
        }
    }

    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserAutomationContext {
    pub start_url: String,
}

impl BrowserAutomationContext {
    pub fn new(start_url: impl Into<String>) -> Self {
        Self {
            start_url: normalize_url(start_url.into()),
        }
    }

    pub fn baidu() -> Self {
        Self::new("https://www.baidu.com")
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct BrowserAutomationContextStore;

impl BrowserAutomationContextStore {
    pub fn set(context: BrowserAutomationContext) {
        ThreadLocalUtil::set(context);
    }

    pub fn set_start_url(start_url: impl Into<String>) {
        Self::set(BrowserAutomationContext::new(start_url));
    }

    pub fn set_baidu() {
        Self::set(BrowserAutomationContext::baidu());
    }

    pub fn get() -> Option<BrowserAutomationContext> {
        ThreadLocalUtil::get::<BrowserAutomationContext>()
    }

    pub fn start_url() -> Option<String> {
        Self::get().map(|context| context.start_url)
    }

    pub fn clear() {
        ThreadLocalUtil::remove::<BrowserAutomationContext>();
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct BrowserAutomation;

impl BrowserAutomation {
    pub fn with_tab<T>(
        url: impl AsRef<str>,
        options: &BrowserAutomationOptions,
        block: impl FnOnce(&Arc<Tab>) -> BrowserAutomationResult<T>,
    ) -> BrowserAutomationResult<T> {
        let browser = connect_browser(options)?;
        let tab = browser
            .new_tab()
            .map_err(|error| BrowserAutomationError::Browser(error.to_string()))?;

        tab.set_default_timeout(Duration::from_millis(options.timeout_ms));
        inject_stealth(tab.as_ref())?;
        tab.navigate_to(url.as_ref())
            .and_then(|tab| tab.wait_until_navigated())
            .map_err(|error| BrowserAutomationError::Browser(error.to_string()))?;

        block(&tab)
    }

    pub fn fill(
        url: impl AsRef<str>,
        fields: &[FormFieldDef],
        options: &BrowserAutomationOptions,
        submit_selectors: Option<&[String]>,
    ) -> BrowserAutomationResult<()> {
        let url = normalize_url(url.as_ref().to_owned());
        Self::with_tab(&url, options, |tab| {
            for field in fields {
                Self::perform_field(tab, &url, field, options)?;
            }

            if let Some(selectors) = submit_selectors {
                let clicked = click_any(tab, selectors, options)?;
                if !clicked {
                    debug_dump(tab, options, "submit")?;
                    return Err(BrowserAutomationError::MissingRequiredField {
                        name: "submit".to_owned(),
                        url: url.clone(),
                    });
                }
            }

            Ok(())
        })
    }

    pub fn fill_steps(
        url: impl AsRef<str>,
        steps: &[Vec<FormFieldDef>],
        options: &BrowserAutomationOptions,
    ) -> BrowserAutomationResult<()> {
        let url = normalize_url(url.as_ref().to_owned());
        Self::with_tab(&url, options, |tab| {
            for (index, fields) in steps.iter().enumerate() {
                if index > 0 {
                    slow_mo(options);
                }
                for field in fields {
                    Self::perform_field(tab, &url, field, options)?;
                }
            }
            Ok(())
        })
    }

    pub fn fill_from_context(
        fields: &[FormFieldDef],
        options: &BrowserAutomationOptions,
        submit_selectors: Option<&[String]>,
    ) -> BrowserAutomationResult<()> {
        let start_url = BrowserAutomationContextStore::start_url()
            .ok_or(BrowserAutomationError::MissingStartUrl)?;
        Self::fill(start_url, fields, options, submit_selectors)
    }

    pub fn run_baidu_search(
        query: impl AsRef<str>,
        options: &BrowserAutomationOptions,
    ) -> BrowserAutomationResult<()> {
        BrowserAutomationContextStore::set_baidu();
        let fields = vec![
            FormFieldDef::input(
                "keyword",
                ["input[name='wd']", "input#kw", "input.s_ipt"],
                query.as_ref(),
            )
            .required(true),
            FormFieldDef::click("search", ["input#su", "button[type='submit']"]).required(true),
        ];
        Self::fill_from_context(&fields, options, None)
    }

    pub fn open_and_hold(
        url: impl AsRef<str>,
        options: &BrowserAutomationOptions,
        hold_for: Duration,
    ) -> BrowserAutomationResult<()> {
        let url = normalize_url(url.as_ref().to_owned());
        Self::with_tab(&url, options, |_| {
            thread::sleep(hold_for);
            Ok(())
        })
    }

    pub fn open_and_hold_from_context(
        options: &BrowserAutomationOptions,
        hold_for: Duration,
    ) -> BrowserAutomationResult<()> {
        let start_url = BrowserAutomationContextStore::start_url()
            .ok_or(BrowserAutomationError::MissingStartUrl)?;
        Self::open_and_hold(start_url, options, hold_for)
    }

    fn perform_field(
        tab: &Arc<Tab>,
        url: &str,
        field: &FormFieldDef,
        options: &BrowserAutomationOptions,
    ) -> BrowserAutomationResult<()> {
        let handled = match field.field_type {
            FieldType::Input => {
                if field.value.is_empty() && !field.required {
                    true
                } else {
                    fill_any(tab, &field.selectors, &field.value, options)?
                }
            }
            FieldType::Click => click_any(tab, &field.selectors, options)?,
            FieldType::Check => check_any(tab, &field.selectors, options)?,
        };

        if handled || !field.required {
            return Ok(());
        }

        debug_dump(tab, options, &field.name)?;
        Err(BrowserAutomationError::MissingRequiredField {
            name: field.name.clone(),
            url: url.to_owned(),
        })
    }
}

fn build_launch_options(
    options: &BrowserAutomationOptions,
) -> BrowserAutomationResult<headless_chrome::LaunchOptions<'static>> {
    let mut builder = LaunchOptionsBuilder::default();
    builder
        .headless(options.effective_headless())
        .sandbox(false)
        .idle_browser_timeout(Duration::from_millis(options.timeout_ms))
        .args(vec![
            OsStr::new("--disable-blink-features=AutomationControlled"),
            OsStr::new("--disable-infobars"),
            OsStr::new("--no-first-run"),
            OsStr::new("--no-default-browser-check"),
            OsStr::new("--disable-component-update"),
            OsStr::new("--disable-background-networking"),
            OsStr::new("--password-store=basic"),
        ]);

    if let Some(path) = options
        .executable_path
        .clone()
        .or_else(resolve_chrome_executable)
    {
        builder.path(Some(path));
    }

    builder
        .build()
        .map_err(|error| BrowserAutomationError::InvalidLaunchOptions(error.to_string()))
}

fn connect_browser(options: &BrowserAutomationOptions) -> BrowserAutomationResult<Browser> {
    match &options.mode {
        BrowserMode::Launch => Browser::new(build_launch_options(options)?)
            .map_err(|error| BrowserAutomationError::Browser(error.to_string())),
        BrowserMode::Cdp(endpoint) => {
            let ws_url = resolve_cdp_websocket_url(endpoint, options.timeout_ms)?;
            Browser::connect_with_timeout(ws_url, Duration::from_millis(options.timeout_ms))
                .map_err(|error| BrowserAutomationError::Browser(error.to_string()))
        }
    }
}

fn resolve_cdp_websocket_url(
    endpoint: &CdpEndpoint,
    timeout_ms: u64,
) -> BrowserAutomationResult<String> {
    match endpoint {
        CdpEndpoint::WebSocket(url) => Ok(url.clone()),
        CdpEndpoint::Http(endpoint) => {
            let endpoint = endpoint.trim_end_matches('/').to_owned();
            match query_cdp_websocket_url(&endpoint, timeout_ms) {
                Ok(ws_url) => Ok(ws_url),
                Err(_) => {
                    ensure_cdp_chrome_running(&endpoint)?;
                    query_cdp_websocket_url(&endpoint, timeout_ms)
                }
            }
        }
    }
}

fn query_cdp_websocket_url(endpoint: &str, timeout_ms: u64) -> BrowserAutomationResult<String> {
    let version_url = format!("{endpoint}/json/version");
    let client = Client::builder()
        .timeout(Duration::from_millis(timeout_ms))
        .build()
        .map_err(|error| BrowserAutomationError::CdpEndpointQuery {
            endpoint: version_url.clone(),
            message: error.to_string(),
        })?;
    let response = client
        .get(&version_url)
        .send()
        .and_then(|response| response.error_for_status())
        .map_err(|error| BrowserAutomationError::CdpEndpointQuery {
            endpoint: version_url.clone(),
            message: error.to_string(),
        })?;
    let payload: Value =
        response
            .json()
            .map_err(|error| BrowserAutomationError::CdpEndpointQuery {
                endpoint: version_url.clone(),
                message: error.to_string(),
            })?;
    payload
        .get("webSocketDebuggerUrl")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or(BrowserAutomationError::MissingCdpWebSocketUrl(version_url))
}

fn ensure_cdp_chrome_running(endpoint: &str) -> BrowserAutomationResult<()> {
    let port = parse_cdp_port(endpoint).unwrap_or(9222);
    let chrome_path =
        resolve_chrome_executable().ok_or(BrowserAutomationError::ChromeExecutableNotFound)?;
    let profile_dir = std::env::temp_dir().join(format!("addzero-browser-cdp-{port}"));
    fs::create_dir_all(&profile_dir)
        .map_err(|error| BrowserAutomationError::ChromeLaunch(error.to_string()))?;

    Command::new(chrome_path)
        .arg(format!("--remote-debugging-port={port}"))
        .arg(format!("--user-data-dir={}", profile_dir.display()))
        .arg("--no-first-run")
        .arg("--no-default-browser-check")
        .arg("--disable-blink-features=AutomationControlled")
        .arg("--disable-infobars")
        .arg("--disable-background-networking")
        .arg("--password-store=basic")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| BrowserAutomationError::ChromeLaunch(error.to_string()))?;

    let deadline = std::time::Instant::now() + Duration::from_secs(15);
    while std::time::Instant::now() < deadline {
        if query_cdp_websocket_url(endpoint, 1_500).is_ok() {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(500));
    }

    Err(BrowserAutomationError::ChromeLaunch(format!(
        "Chrome was started but CDP endpoint did not become ready at {endpoint} within 15s"
    )))
}

pub fn parse_cdp_port(endpoint: &str) -> Option<u16> {
    let after_scheme = endpoint
        .split_once("://")
        .map(|(_, rest)| rest)
        .unwrap_or(endpoint);
    let host_port = after_scheme.split('/').next().unwrap_or(after_scheme);
    host_port
        .rsplit_once(':')
        .and_then(|(_, port)| port.parse::<u16>().ok())
}

fn resolve_chrome_executable() -> Option<PathBuf> {
    const CANDIDATES: &[&str] = &[
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/Applications/Chromium.app/Contents/MacOS/Chromium",
        "/usr/bin/google-chrome",
        "/usr/bin/google-chrome-stable",
        "/usr/bin/chromium-browser",
        "/usr/bin/chromium",
        "/snap/bin/chromium",
        "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
        "C:\\Program Files (x86)\\Google\\Chrome\\Application\\chrome.exe",
        "C:\\Program Files\\Microsoft\\Edge\\Application\\msedge.exe",
        "C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe",
    ];

    CANDIDATES
        .iter()
        .map(PathBuf::from)
        .find(|path| path.exists())
}

fn inject_stealth(tab: &Tab) -> BrowserAutomationResult<()> {
    tab.evaluate(
        r#"
        (() => {
          Object.defineProperty(navigator, "webdriver", { get: () => undefined });
          Object.defineProperty(navigator, "plugins", { get: () => [1, 2, 3, 4, 5] });
          Object.defineProperty(navigator, "languages", { get: () => ["zh-CN", "zh", "en-US", "en"] });
          window.chrome = window.chrome || { runtime: {} };
        })()
        "#,
        false,
    )
    .map_err(|error| BrowserAutomationError::Browser(error.to_string()))?;
    Ok(())
}

fn fill_any(
    tab: &Arc<Tab>,
    selectors: &[String],
    value: &str,
    options: &BrowserAutomationOptions,
) -> BrowserAutomationResult<bool> {
    for selector in selectors {
        match find_selector(tab, selector, options)? {
            Some(found) => {
                found
                    .click()
                    .map_err(|error| BrowserAutomationError::Browser(error.to_string()))?;
                found
                    .focus()
                    .map_err(|error| BrowserAutomationError::Browser(error.to_string()))?;
                let _ = found.call_js_fn(
                    "function() { if ('value' in this) { this.value = ''; } }",
                    vec![],
                    false,
                );
                found
                    .type_into(value)
                    .map_err(|error| BrowserAutomationError::Browser(error.to_string()))?;
                slow_mo(options);
                return Ok(true);
            }
            None => continue,
        }
    }

    Ok(false)
}

fn click_any(
    tab: &Arc<Tab>,
    selectors: &[String],
    options: &BrowserAutomationOptions,
) -> BrowserAutomationResult<bool> {
    for selector in selectors {
        match find_selector(tab, selector, options)? {
            Some(found) => {
                found
                    .click()
                    .map_err(|error| BrowserAutomationError::Browser(error.to_string()))?;
                slow_mo(options);
                return Ok(true);
            }
            None => continue,
        }
    }

    Ok(false)
}

fn check_any(
    tab: &Arc<Tab>,
    selectors: &[String],
    options: &BrowserAutomationOptions,
) -> BrowserAutomationResult<bool> {
    for selector in selectors {
        match find_selector(tab, selector, options)? {
            Some(found) => {
                found
                    .click()
                    .map_err(|error| BrowserAutomationError::Browser(error.to_string()))?;
                let _ = found.call_js_fn(
                    "function() { if ('checked' in this) { this.checked = true; } }",
                    vec![],
                    false,
                );
                slow_mo(options);
                return Ok(true);
            }
            None => continue,
        }
    }

    Ok(false)
}

fn find_selector<'a>(
    tab: &'a Arc<Tab>,
    selector: &str,
    options: &BrowserAutomationOptions,
) -> BrowserAutomationResult<Option<headless_chrome::Element<'a>>> {
    let timeout = Duration::from_millis(options.timeout_ms);
    let found = if let Some(query) = selector.strip_prefix("label:") {
        tab.wait_for_xpath_with_custom_timeout(&label_xpath(query), timeout)
    } else if let Some(query) = selector.strip_prefix("placeholder:") {
        tab.wait_for_element_with_custom_timeout(&placeholder_css(query), timeout)
    } else if let Some(query) = selector.strip_prefix("role:") {
        tab.wait_for_xpath_with_custom_timeout(&role_xpath(query), timeout)
    } else {
        tab.wait_for_element_with_custom_timeout(selector, timeout)
    };

    match found {
        Ok(element) => Ok(Some(element)),
        Err(_) => Ok(None),
    }
}

fn label_xpath(label: &str) -> String {
    let label = xpath_literal(label);
    format!(
        "//*[@aria-label={label}] | //label[normalize-space()={label}]/following::*[self::input or self::textarea or self::select][1]"
    )
}

fn placeholder_css(placeholder: &str) -> String {
    format!(r#"[placeholder="{}"]"#, placeholder.replace('"', "\\\""))
}

fn role_xpath(raw: &str) -> String {
    let mut parts = raw.splitn(2, ':');
    let role = parts.next().unwrap_or_default().trim();
    let role_lit = xpath_literal(role);
    match parts
        .next()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(name) => {
            let name_lit = xpath_literal(name);
            format!(
                "//*[@role={role_lit} and (@aria-label={name_lit} or normalize-space()={name_lit})] | //{role}[normalize-space()={name_lit} or @aria-label={name_lit}]"
            )
        }
        None => format!("//*[@role={role_lit}] | //{role}"),
    }
}

fn xpath_literal(value: &str) -> String {
    if !value.contains('\'') {
        return format!("'{value}'");
    }

    let parts = value
        .split('\'')
        .map(|segment| format!("'{segment}'"))
        .collect::<Vec<_>>();
    format!("concat({})", parts.join(", \"'\", "))
}

fn debug_dump(
    tab: &Arc<Tab>,
    options: &BrowserAutomationOptions,
    name: &str,
) -> BrowserAutomationResult<()> {
    if !options.debug && options.artifacts_dir.is_none() {
        return Ok(());
    }

    let directory = options
        .artifacts_dir
        .clone()
        .unwrap_or_else(|| std::env::temp_dir().join("addzero-browser-automation"));
    fs::create_dir_all(&directory).map_err(|source| BrowserAutomationError::ArtifactIo {
        path: directory.clone(),
        source,
    })?;

    let base = directory.join(format!(
        "{}-{}",
        timestamp_millis(),
        sanitize_file_stem(name)
    ));
    let screenshot_path = base.with_extension("png");
    let html_path = base.with_extension("html");

    let screenshot = tab
        .capture_screenshot(CaptureScreenshotFormatOption::Png, None, None, true)
        .map_err(to_browser_error)?;
    fs::write(&screenshot_path, screenshot).map_err(|source| {
        BrowserAutomationError::ArtifactIo {
            path: screenshot_path,
            source,
        }
    })?;

    let html = tab.get_content().map_err(to_browser_error)?;
    fs::write(&html_path, html).map_err(|source| BrowserAutomationError::ArtifactIo {
        path: html_path,
        source,
    })?;

    Ok(())
}

fn sanitize_file_stem(name: &str) -> String {
    name.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn timestamp_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn normalize_url(url: String) -> String {
    let trimmed = url.trim();
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_owned()
    } else {
        format!("https://{trimmed}")
    }
}

pub fn normalize_cdp_http_url(endpoint: impl AsRef<str>) -> String {
    let endpoint = endpoint.as_ref().trim();
    if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
        endpoint.trim_end_matches('/').to_owned()
    } else {
        format!("http://{}", endpoint.trim_end_matches('/'))
    }
}

fn slow_mo(options: &BrowserAutomationOptions) {
    if options.slow_mo_ms > 0 {
        thread::sleep(Duration::from_millis(options.slow_mo_ms));
    }
}

fn to_browser_error(error: impl ToString) -> BrowserAutomationError {
    BrowserAutomationError::Browser(error.to_string())
}
