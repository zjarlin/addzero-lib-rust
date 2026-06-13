//! Clash/Mihomo 专用解析、配置生成和本地进程辅助工具。

automod::dir!(pub "src/clash");

use crate::types::ProxyNode;
use anyhow::Result;

/// 默认 Clash mixed HTTP/SOCKS 监听端口。
pub const DEFAULT_MIXED_PORT: u16 = config::DEFAULT_MIXED_PORT;

/// Clash 配置文档类型。
pub type ClashConfig = config::ClashConfig;
/// Clash 代理组配置类型。
pub type ProxyGroup = config::ProxyGroup;
/// 由本地 Clash/Mihomo 进程承载的住宅代理句柄。
pub type ResidentialProxy = residential::ResidentialProxy;
/// 启动本地住宅代理的配置。
pub type ResidentialProxyConfig = residential::ResidentialProxyConfig;

/// 为指定节点生成最小 Clash YAML 配置。
pub fn generate_clash_config(node: &ProxyNode, mixed_port: u16) -> Result<String> {
    config::generate_clash_config(node, mixed_port)
}

/// 获取订阅、测速并生成最快节点对应的 Clash YAML。
pub async fn select_fastest(url: &str, concurrency: usize) -> Result<String> {
    config::select_fastest(url, concurrency).await
}

/// 从 Clash YAML 中解析受支持的代理节点。
pub fn parse_clash_yaml(input: &str) -> Result<Vec<ProxyNode>> {
    yaml::parse_clash_yaml(input)
}
