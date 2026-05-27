use crate::fetcher::fetch_and_parse;
use crate::selector::select_fastest_node;
use crate::speedtest::batch_speed_test;
use crate::types::{DEFAULT_SPEEDTEST_TIMEOUT, ProxyNode, ProxyResult};
use az_derive_aliases::{apply, serde_eq, serde_partial_eq};
use serde_yaml::{Mapping, Number, Value};

/// [`select_fastest`] 使用的默认 Clash mixed HTTP/SOCKS 监听端口。
pub const DEFAULT_MIXED_PORT: u16 = 7890;

/// 为选中节点生成的最小 Clash 配置文档。
#[apply(serde_partial_eq)]
pub struct ClashConfig {
    /// HTTP/SOCKS 混合监听端口。
    #[serde(rename = "mixed-port")]
    pub mixed_port: u16,
    /// Clash 是否监听局域网接口。
    #[serde(rename = "allow-lan")]
    pub allow_lan: bool,
    /// Clash 路由模式。
    pub mode: String,
    /// Clash 日志级别。
    #[serde(rename = "log-level")]
    pub log_level: String,
    /// 生成配置中包含的代理定义。
    pub proxies: Vec<Value>,
    /// 生成配置中包含的代理组。
    #[serde(rename = "proxy-groups")]
    pub proxy_groups: Vec<ProxyGroup>,
    /// Clash 路由规则。
    pub rules: Vec<String>,
}

impl ClashConfig {
    /// 构造只包含一个代理节点的最小 rule-mode Clash 配置。
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

/// Clash 代理组配置项。
#[apply(serde_eq)]
pub struct ProxyGroup {
    /// 代理组名称。
    pub name: String,
    /// Clash 代理组类型，例如 `select`。
    #[serde(rename = "type")]
    pub group_type: String,
    /// 属于该代理组的代理名称列表。
    pub proxies: Vec<String>,
}

/// 生成仅包含 `node` 的最小 Clash YAML 配置。
///
/// # Errors
///
/// 当生成的配置无法序列化为 YAML 时返回错误。
pub fn generate_clash_config(node: &ProxyNode, mixed_port: u16) -> ProxyResult<String> {
    let proxy = normalized_proxy_value(node);
    let config = ClashConfig::minimal(mixed_port, proxy, node.name.clone());
    Ok(serde_yaml::to_string(&config)?)
}

/// 获取订阅、解析节点、测速、选择最快节点，并返回 Clash YAML。
///
/// 使用 [`DEFAULT_SPEEDTEST_TIMEOUT`] 和 [`DEFAULT_MIXED_PORT`]。
///
/// # Errors
///
/// 当获取/解析失败、所有测速都失败，或选中配置无法序列化时返回错误。
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
