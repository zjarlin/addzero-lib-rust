//! Clash/Mihomo-specific parsing, config generation, and local process helpers.

automod::dir!(pub "src/clash");

use crate::types::{ProxyNode, ProxyResult};

pub const DEFAULT_MIXED_PORT: u16 = config::DEFAULT_MIXED_PORT;

pub type ClashConfig = config::ClashConfig;
pub type ProxyGroup = config::ProxyGroup;
pub type ResidentialProxy = residential::ResidentialProxy;
pub type ResidentialProxyConfig = residential::ResidentialProxyConfig;

pub fn generate_clash_config(node: &ProxyNode, mixed_port: u16) -> ProxyResult<String> {
    config::generate_clash_config(node, mixed_port)
}

pub async fn select_fastest(url: &str, concurrency: usize) -> ProxyResult<String> {
    config::select_fastest(url, concurrency).await
}

pub fn parse_clash_yaml(input: &str) -> ProxyResult<Vec<ProxyNode>> {
    yaml::parse_clash_yaml(input)
}
