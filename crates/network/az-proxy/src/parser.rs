//! 代理订阅解析分发层，负责识别 Clash YAML、明文 URI 列表和 base64 URI 列表。

use crate::clash::parse_clash_yaml;
use crate::types::{ProxyError, ProxyNode, ProxyResult};
use base64::Engine;

automod::dir!(pub "src/parser");

/// 将单个受支持的代理 URI 解析为代理节点。
///
/// 支持的 URI scheme 包括 `ss`、`vmess`、`vless`、`trojan`、
/// `hysteria2`、`hy2`、`tuic` 和 `wireguard`。
///
/// # Errors
///
/// 当 scheme 不受支持，或 URI 缺少生成 Clash 兼容代理项所需的数据时返回错误。
pub fn parse_proxy_uri(input: &str) -> ProxyResult<ProxyNode> {
    uri::parse_proxy_uri(input)
}

/// 解析按行分隔的代理 URI 订阅。
///
/// 空行和不包含受支持 URI scheme 的行会被忽略。
///
/// # Errors
///
/// 当某个可识别 URI 行格式错误，或最终没有任何可用节点时返回错误。
pub fn parse_uri_lines(input: &str) -> ProxyResult<Vec<ProxyNode>> {
    uri::parse_uri_lines(input)
}

/// 将订阅文本解析为受支持的代理节点。
///
/// `content_type` 只是辅助提示；函数仍会检查正文内容，因此响应头错误的订阅也能被识别。
///
/// # Errors
///
/// 当正文为空、疑似 YAML 但 YAML 非法、URI 数据非法，或没有任何可用节点时返回错误。
pub fn parse_subscription(body: &str, content_type: Option<&str>) -> ProxyResult<Vec<ProxyNode>> {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return Err(ProxyError::NoUsableNodes);
    }

    if looks_like_clash_yaml(trimmed, content_type) {
        return parse_clash_yaml(trimmed);
    }

    if contains_supported_uri(trimmed) {
        return parse_uri_lines(trimmed);
    }

    if let Some(decoded) = decode_base64_text(trimmed) {
        let decoded = decoded.trim();
        if looks_like_clash_yaml(decoded, None) {
            return parse_clash_yaml(decoded);
        }
        return parse_uri_lines(decoded);
    }

    let yaml_result = parse_clash_yaml(trimmed);
    if yaml_result.as_ref().is_ok_and(|nodes| !nodes.is_empty()) {
        return yaml_result;
    }

    Err(ProxyError::NoUsableNodes)
}

fn looks_like_clash_yaml(body: &str, content_type: Option<&str>) -> bool {
    let content_type_is_yaml = content_type.is_some_and(|value| {
        let value = value.to_ascii_lowercase();
        value.contains("yaml") || value.contains("yml")
    });

    content_type_is_yaml
        || body.starts_with("proxies:")
        || body.contains("\nproxies:")
        || body.contains("\r\nproxies:")
}

fn contains_supported_uri(body: &str) -> bool {
    body.lines().any(|line| {
        let line = line.trim_start();
        matches!(
            line.split_once("://").map(|(scheme, _)| scheme),
            Some("ss" | "vmess" | "vless" | "trojan" | "hysteria2" | "hy2" | "tuic" | "wireguard")
        )
    })
}

pub(crate) fn decode_base64_text(input: &str) -> Option<String> {
    let compact: String = input.chars().filter(|ch| !ch.is_whitespace()).collect();
    if compact.is_empty() {
        return None;
    }

    [
        base64::engine::general_purpose::STANDARD,
        base64::engine::general_purpose::STANDARD_NO_PAD,
        base64::engine::general_purpose::URL_SAFE,
        base64::engine::general_purpose::URL_SAFE_NO_PAD,
    ]
    .iter()
    .find_map(|engine| {
        engine
            .decode(compact.as_bytes())
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
    })
}
