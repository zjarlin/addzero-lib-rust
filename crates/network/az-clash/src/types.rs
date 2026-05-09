use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::time::Duration;
use thiserror::Error;

/// Default Clash mixed HTTP/SOCKS listen port used by [`crate::select_fastest`].
pub const DEFAULT_MIXED_PORT: u16 = 7890;

/// Default maximum number of concurrent TCP latency checks.
pub const DEFAULT_SPEEDTEST_CONCURRENCY: usize = 10;

/// Default per-node TCP connection timeout.
pub const DEFAULT_SPEEDTEST_TIMEOUT: Duration = Duration::from_secs(5);

/// Result alias used by all fallible `az-clash` operations.
pub type ClashResult<T> = Result<T, ClashError>;

/// Errors returned by subscription fetching, parsing, selection, and config generation.
#[derive(Debug, Error)]
pub enum ClashError {
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
}

/// Supported proxy node types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProxyType {
    /// Shadowsocks proxy node.
    Ss,
    /// VMess proxy node.
    Vmess,
    /// VLESS proxy node.
    Vless,
    /// Trojan proxy node.
    Trojan,
    /// Hysteria2 or `hy2` proxy node.
    Hysteria2,
    /// TUIC proxy node.
    Tuic,
    /// WireGuard proxy node.
    Wireguard,
}

impl ProxyType {
    /// Returns the Clash YAML `type` string for this proxy type.
    pub fn as_clash_str(self) -> &'static str {
        match self {
            Self::Ss => "ss",
            Self::Vmess => "vmess",
            Self::Vless => "vless",
            Self::Trojan => "trojan",
            Self::Hysteria2 => "hysteria2",
            Self::Tuic => "tuic",
            Self::Wireguard => "wireguard",
        }
    }

    /// Parses a Clash YAML `type` string into a supported proxy type.
    pub fn from_clash_type(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "ss" | "shadowsocks" => Some(Self::Ss),
            "vmess" => Some(Self::Vmess),
            "vless" => Some(Self::Vless),
            "trojan" => Some(Self::Trojan),
            "hysteria2" | "hy2" => Some(Self::Hysteria2),
            "tuic" => Some(Self::Tuic),
            "wireguard" => Some(Self::Wireguard),
            _ => None,
        }
    }

    /// Parses a URI scheme into a supported proxy type.
    pub fn from_uri_scheme(value: &str) -> Option<Self> {
        Self::from_clash_type(value)
    }
}

/// A normalized proxy node parsed from Clash YAML or a proxy URI.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

/// A minimal Clash config document generated for a selected node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClashConfig {
    /// Mixed HTTP/SOCKS listen port.
    #[serde(rename = "mixed-port")]
    pub mixed_port: u16,
    /// Whether Clash should listen on LAN interfaces.
    #[serde(rename = "allow-lan")]
    pub allow_lan: bool,
    /// Clash routing mode.
    pub mode: String,
    /// Clash log level.
    #[serde(rename = "log-level")]
    pub log_level: String,
    /// Proxy definitions included in the generated config.
    pub proxies: Vec<Value>,
    /// Proxy groups included in the generated config.
    #[serde(rename = "proxy-groups")]
    pub proxy_groups: Vec<ProxyGroup>,
    /// Clash routing rules.
    pub rules: Vec<String>,
}

impl ClashConfig {
    /// Builds a minimal rule-mode Clash config containing exactly one proxy node.
    pub fn minimal(mixed_port: u16, proxy: Value, proxy_name: impl Into<String>) -> Self {
        let proxy_name = proxy_name.into();
        Self {
            mixed_port,
            allow_lan: false,
            mode: "rule".to_owned(),
            log_level: "info".to_owned(),
            proxies: vec![proxy],
            proxy_groups: vec![ProxyGroup {
                name: "PROXY".to_owned(),
                group_type: "select".to_owned(),
                proxies: vec![proxy_name],
            }],
            rules: vec![
                "DOMAIN-SUFFIX,local,DIRECT".to_owned(),
                "IP-CIDR,127.0.0.0/8,DIRECT".to_owned(),
                "IP-CIDR,10.0.0.0/8,DIRECT".to_owned(),
                "IP-CIDR,172.16.0.0/12,DIRECT".to_owned(),
                "IP-CIDR,192.168.0.0/16,DIRECT".to_owned(),
                "GEOIP,CN,DIRECT".to_owned(),
                "MATCH,PROXY".to_owned(),
            ],
        }
    }
}

/// A Clash proxy group entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProxyGroup {
    /// Proxy group name.
    pub name: String,
    /// Clash group type such as `select`.
    #[serde(rename = "type")]
    pub group_type: String,
    /// Names of proxies that belong to this group.
    pub proxies: Vec<String>,
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
