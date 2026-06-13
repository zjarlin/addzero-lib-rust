use az_derive_aliases::{apply, serde_code_props_enum, serde_eq, serde_partial_eq};
use serde_yaml::Value;
use std::time::Duration;

/// TCP 延迟测试的默认并发数上限。
pub const DEFAULT_SPEEDTEST_CONCURRENCY: usize = 10;

/// 单个节点 TCP 连接测试的默认超时时间。
pub const DEFAULT_SPEEDTEST_TIMEOUT: Duration = Duration::from_secs(5);

/// 当前支持的代理节点类型。
#[apply(serde_code_props_enum)]
pub enum ProxyType {
    /// Shadowsocks 代理节点。
    #[strum(serialize = "ss", serialize = "shadowsocks", props(clash = "ss"))]
    Ss,
    /// VMess 代理节点。
    #[strum(props(clash = "vmess"))]
    Vmess,
    /// VLESS 代理节点。
    #[strum(props(clash = "vless"))]
    Vless,
    /// Trojan 代理节点。
    #[strum(props(clash = "trojan"))]
    Trojan,
    /// Hysteria2 或 `hy2` 代理节点。
    #[strum(serialize = "hysteria2", serialize = "hy2", props(clash = "hysteria2"))]
    Hysteria2,
    /// TUIC 代理节点。
    #[strum(props(clash = "tuic"))]
    Tuic,
    /// WireGuard 代理节点。
    #[strum(props(clash = "wireguard"))]
    Wireguard,
}

impl ProxyType {
    /// 返回 Clash YAML 中使用的规范 `type` 字符串。
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

    /// 将 Clash YAML 的 `type` 字符串解析为受支持的代理类型。
    pub fn from_clash_type(value: &str) -> Option<Self> {
        value.trim().to_ascii_lowercase().parse().ok()
    }

    /// 将 URI scheme 解析为受支持的代理类型。
    pub fn from_uri_scheme(value: &str) -> Option<Self> {
        Self::from_clash_type(value)
    }
}

/// 从 Clash YAML 或代理 URI 归一化后的代理节点。
#[apply(serde_partial_eq)]
pub struct ProxyNode {
    /// 来自 YAML 或 URI fragment 的可读节点名称。
    pub name: String,
    /// 归一化后的代理类型。
    pub node_type: ProxyType,
    /// 代理服务器主机名或 IP 地址。
    pub server: String,
    /// 代理 TCP 端口。
    pub port: u16,
    /// 从节点名称中尽力解析出的国家或地区代码。
    pub country: Option<String>,
    /// 原始 YAML 代理配置；若来源是 URI，则为生成的 Clash 兼容 YAML 值。
    pub raw: Value,
}

impl ProxyNode {
    /// 创建代理节点，并根据名称推断国家或地区代码。
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

/// 单个 TCP 连接延迟测试结果。
#[apply(serde_eq)]
pub struct SpeedTestResult {
    /// 被测节点在原始节点列表中的下标。
    pub node_index: usize,
    /// 测试成功时的 TCP 连接耗时，单位毫秒。
    pub latency_ms: Option<u128>,
    /// TCP 连接是否在超时前完成。
    pub success: bool,
    /// 测试失败时的错误信息。
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
