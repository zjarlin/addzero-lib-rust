use crate::fetcher::fetch_and_parse;
use crate::selector::select_fastest_node;
use crate::speedtest::batch_speed_test;
use crate::types::{DEFAULT_SPEEDTEST_TIMEOUT, ProxyNode, ProxyResult};
use az_derive_aliases::{apply, serde_eq, serde_partial_eq};
use serde_yaml::{Mapping, Number, Value};

/// Default Clash mixed HTTP/SOCKS listen port used by [`select_fastest`].
pub const DEFAULT_MIXED_PORT: u16 = 7890;

/// A minimal Clash config document generated for a selected node.
#[apply(serde_partial_eq)]
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
#[apply(serde_eq)]
pub struct ProxyGroup {
    /// Proxy group name.
    pub name: String,
    /// Clash group type such as `select`.
    #[serde(rename = "type")]
    pub group_type: String,
    /// Names of proxies that belong to this group.
    pub proxies: Vec<String>,
}

/// Generates a minimal Clash YAML config containing `node` as the only proxy.
///
/// # Errors
///
/// Returns an error when the generated config cannot be serialized as YAML.
pub fn generate_clash_config(node: &ProxyNode, mixed_port: u16) -> ProxyResult<String> {
    let proxy = normalized_proxy_value(node);
    let config = ClashConfig::minimal(mixed_port, proxy, node.name.clone());
    Ok(serde_yaml::to_string(&config)?)
}

/// Fetches, parses, speed-tests, selects the fastest node, and returns Clash YAML.
///
/// Uses [`DEFAULT_SPEEDTEST_TIMEOUT`] and [`DEFAULT_MIXED_PORT`].
///
/// # Errors
///
/// Returns an error when fetching/parsing fails, every speed test fails, or the
/// selected config cannot be serialized.
pub async fn select_fastest(url: &str, concurrency: usize) -> ProxyResult<String> {
    let nodes = fetch_and_parse(url).await?;
    let results = batch_speed_test(&nodes, concurrency, DEFAULT_SPEEDTEST_TIMEOUT).await;
    let node = select_fastest_node(&nodes, &results)?;
    generate_clash_config(node, DEFAULT_MIXED_PORT)
}

fn normalized_proxy_value(node: &ProxyNode) -> Value {
    let Value::Mapping(mut map) = node.raw.clone() else {
        return fallback_proxy_value(node);
    };

    ensure_string(&mut map, "name", &node.name);
    ensure_string(&mut map, "type", node.node_type.as_clash_str());
    ensure_string(&mut map, "server", &node.server);
    ensure_port(&mut map, node.port);
    Value::Mapping(map)
}

fn fallback_proxy_value(node: &ProxyNode) -> Value {
    let mut map = Mapping::new();
    ensure_string(&mut map, "name", &node.name);
    ensure_string(&mut map, "type", node.node_type.as_clash_str());
    ensure_string(&mut map, "server", &node.server);
    ensure_port(&mut map, node.port);
    Value::Mapping(map)
}

fn ensure_string(map: &mut Mapping, key: &str, value: &str) {
    map.insert(
        Value::String(key.to_owned()),
        Value::String(value.to_owned()),
    );
}

fn ensure_port(map: &mut Mapping, port: u16) {
    map.insert(
        Value::String("port".to_owned()),
        Value::Number(Number::from(port)),
    );
}
