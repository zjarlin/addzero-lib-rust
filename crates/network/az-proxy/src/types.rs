use az_derive_aliases::{apply, error, serde_code_enum, serde_eq, serde_partial_eq};
use serde_yaml::Value;
use std::time::Duration;

/// Default maximum number of concurrent TCP latency checks.
pub const DEFAULT_SPEEDTEST_CONCURRENCY: usize = 10;

/// Default per-node TCP connection timeout.
pub const DEFAULT_SPEEDTEST_TIMEOUT: Duration = Duration::from_secs(5);

/// Result alias used by all fallible `az-proxy` operations.
pub type ProxyResult<T> = Result<T, ProxyError>;

/// Errors returned by subscription fetching, parsing, selection, and config generation.
#[apply(error)]
pub enum ProxyError {
    /// The HTTP request failed or returned an unsuccessful status.
    #[error("http request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// The subscription body could not be parsed as YAML.
    #[error("yaml parse failed: {0}")]
    Yaml(#[from] serde_yaml::Error),

    /// A `vmess://` JSON payload could not be parsed.
    #[error("json parse failed: {0}")]
    Json(#[from] serde_json::Error),

    /// A base64 subscription or URI payload could not be decoded.
    #[error("base64 decode failed: {0}")]
    Base64(#[from] base64::DecodeError),

    /// A proxy URI was not a valid URL for its scheme.
    #[error("url parse failed: {0}")]
    Url(#[from] url::ParseError),

    /// A required field was absent from a proxy definition.
    #[error("missing required field `{0}`")]
    MissingField(&'static str),

    /// A proxy type or URI scheme is not supported by this crate.
    #[error("unsupported proxy type `{0}`")]
    UnsupportedProxyType(String),

    /// A proxy port was absent, out of range, or not numeric.
    #[error("invalid proxy port `{0}`")]
    InvalidPort(String),

    /// A proxy URI had the expected scheme but invalid structure.
    #[error("invalid proxy uri: {0}")]
    InvalidUri(String),

    /// The subscription body did not contain usable proxy nodes.
    #[error("subscription did not contain usable proxy nodes")]
    NoUsableNodes,

    /// No speed test result completed successfully.
    #[error("no successful speed test result")]
    NoSuccessfulSpeedTest,

    /// Could not locate a Clash/Mihomo binary on the system.
    #[error("could not find Clash/Mihomo binary; set CLASH_BINARY env var")]
    ClashBinaryNotFound,

    /// Clash process management error.
    #[error("clash process error: {0}")]
    ClashProcess(String),
}

/// Supported proxy node types.
#[apply(serde_code_enum)]
pub enum ProxyType {
    /// Shadowsocks proxy node.
    #[strum(serialize = "ss", serialize = "shadowsocks")]
    Ss,
    /// VMess proxy node.
    Vmess,
    /// VLESS proxy node.
    Vless,
    /// Trojan proxy node.
    Trojan,
    /// Hysteria2 or `hy2` proxy node.
    #[strum(serialize = "hysteria2", serialize = "hy2")]
    Hysteria2,
    /// TUIC proxy node.
    Tuic,
    /// WireGuard proxy node.
    Wireguard,
}

impl ProxyType {
    /// Returns the Clash YAML `type` string for this proxy type.
    pub fn as_clash_str(self) -> &'static str {
        self.code()
    }

    /// Parses a Clash YAML `type` string into a supported proxy type.
    pub fn from_clash_type(value: &str) -> Option<Self> {
        value.trim().to_ascii_lowercase().parse().ok()
    }

    /// Parses a URI scheme into a supported proxy type.
    pub fn from_uri_scheme(value: &str) -> Option<Self> {
        Self::from_clash_type(value)
    }
}

/// A normalized proxy node parsed from Clash YAML or a proxy URI.
#[apply(serde_partial_eq)]
pub struct ProxyNode {
    /// Human-readable node name from YAML or URI fragment.
    pub name: String,
    /// Normalized proxy type.
    pub node_type: ProxyType,
    /// Proxy server hostname or IP address.
    pub server: String,
    /// Proxy TCP port.
    pub port: u16,
    /// Best-effort country or region code parsed from the node name.
    pub country: Option<String>,
    /// Original YAML proxy value, or a generated Clash-compatible YAML value for URI inputs.
    pub raw: Value,
}

impl ProxyNode {
    /// Creates a proxy node and derives its country or region from the name.
    pub fn new(
        name: impl Into<String>,
        node_type: ProxyType,
        server: impl Into<String>,
        port: u16,
        raw: Value,
    ) -> Self {
        let name = name.into();
        Self {
            country: country_from_node_name(&name),
            name,
            node_type,
            server: server.into(),
            port,
            raw,
        }
    }
}

/// Result of one TCP connection latency test.
#[apply(serde_eq)]
pub struct SpeedTestResult {
    /// Index of the tested node in the original node slice.
    pub node_index: usize,
    /// TCP connect latency in milliseconds when the test succeeded.
    pub latency_ms: Option<u128>,
    /// Whether the TCP connection completed before timeout.
    pub success: bool,
    /// Error message when the test failed.
    pub error_msg: Option<String>,
}

pub(crate) fn country_from_node_name(name: &str) -> Option<String> {
    country_from_flag(name).or_else(|| country_from_keywords(name))
}

fn country_from_flag(name: &str) -> Option<String> {
    let mut chars = name.chars();
    while let Some(first) = chars.next() {
        if !is_regional_indicator(first) {
            continue;
        }

        let second = chars.next()?;

        if !is_regional_indicator(second) {
            continue;
        }

        let first = regional_indicator_to_ascii(first)?;
        let second = regional_indicator_to_ascii(second)?;
        return Some(format!("{first}{second}"));
    }
    None
}

fn is_regional_indicator(ch: char) -> bool {
    ('\u{1F1E6}'..='\u{1F1FF}').contains(&ch)
}

fn regional_indicator_to_ascii(ch: char) -> Option<char> {
    let offset = u32::from(ch).checked_sub(0x1F1E6)?;
    char::from_u32(u32::from(b'A') + offset)
}

fn country_from_keywords(name: &str) -> Option<String> {
    const KEYWORDS: [(&str, &str); 22] = [
        ("香港", "HK"),
        ("hong kong", "HK"),
        ("hk", "HK"),
        ("台湾", "TW"),
        ("taiwan", "TW"),
        ("tw", "TW"),
        ("日本", "JP"),
        ("japan", "JP"),
        ("jp", "JP"),
        ("新加坡", "SG"),
        ("singapore", "SG"),
        ("sg", "SG"),
        ("美国", "US"),
        ("united states", "US"),
        ("usa", "US"),
        ("us", "US"),
        ("韩国", "KR"),
        ("korea", "KR"),
        ("kr", "KR"),
        ("英国", "GB"),
        ("united kingdom", "GB"),
        ("uk", "GB"),
    ];

    let lowercase = name.to_ascii_lowercase();
    KEYWORDS
        .iter()
        .find_map(|(needle, country)| lowercase.contains(needle).then(|| (*country).to_owned()))
}
